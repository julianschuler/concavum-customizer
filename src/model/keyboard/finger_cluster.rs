use fidget::context::Tree;
use glam::{DAffine3, DMat3, DVec2, DVec3};

use crate::{
    config::{Keyboard, EPSILON},
    model::{
        geometry::{Line, Plane, Project},
        key_positions::{Column, ColumnType, Columns},
        keyboard::{Bounds, InsertHolder},
        primitives::{Csg, RoundedCsg, SimplePolygon},
        util::{corner_point, prism_from_projected_points, side_point, Side, SideX, SideY},
    },
};

pub struct FingerCluster {
    pub cluster: Tree,
    pub key_clearance: Tree,
    pub insert_holders: Tree,
    pub bounds: Bounds,
}

impl FingerCluster {
    pub fn new(columns: &Columns, config: &Keyboard) -> Self {
        let outline_points = columns.outline_points();
        let cluster_height = columns.max_z() + columns.key_clearance.length();
        let circumference_distance = config.circumference_distance.into();

        let bounds = Bounds::from_outline_points_and_height(
            &outline_points,
            cluster_height,
            circumference_distance,
        );
        let insert_holders = Self::insert_holders(
            &outline_points,
            config,
            columns.len(),
            columns.first().len(),
        );

        let outline: Tree = SimplePolygon::new(outline_points).into();
        let cluster_outline = outline.offset(circumference_distance);
        let cluster = cluster_outline.extrude(-cluster_height, cluster_height);

        let clearance = ClearanceBuilder::new(columns, &bounds).build();
        let cluster = cluster.rounded_difference(clearance, config.rounding_radius);

        let key_clearance = outline.extrude(-cluster_height, cluster_height);
        let insert_holders = insert_holders.intersection(cluster_outline);

        Self {
            cluster,
            key_clearance,
            insert_holders,
            bounds,
        }
    }

    fn insert_holders(
        outline_points: &[DVec2],
        config: &Keyboard,
        columns: usize,
        rows: usize,
    ) -> Tree {
        let first_index = columns - 1;
        let second_index = first_index + rows;
        let third_index = second_index + columns;

        [
            InsertHolder::from_outline_points(outline_points, first_index, config).into(),
            InsertHolder::from_outline_points(outline_points, second_index, config).into(),
            InsertHolder::from_outline_points(outline_points, third_index, config).into(),
        ]
        .into_iter()
        .reduce(|holders: Tree, holder| holders.union(holder))
        .expect("there is more than one insert holder for the finger cluster")
    }
}

struct ClearanceBuilder<'a> {
    columns: &'a Columns,
    bounds: &'a Bounds,
    support_planes: SupportPlanes,
}

impl<'a> ClearanceBuilder<'a> {
    fn new(columns: &'a Columns, bounds: &'a Bounds) -> Self {
        let support_planes = SupportPlanes::from_columns(columns);

        Self {
            columns,
            bounds,
            support_planes,
        }
    }

    fn build(self) -> Tree {
        let columns = self.columns;
        let first = columns.first();
        let last = columns.last();

        let neighbor = match first.column_type {
            ColumnType::Normal => None,
            ColumnType::Side => columns.get(1),
        };
        let left_clearance = self.side_column_clearance(first, neighbor, SideX::Left);

        let neighbor = match last.column_type {
            ColumnType::Normal => None,
            ColumnType::Side => columns.get(columns.len() - 2),
        };
        let right_clearance = self.side_column_clearance(last, neighbor, SideX::Right);

        let mut clearance = left_clearance.union(right_clearance);

        if let Some(columns) = columns.get(1..columns.len() - 1) {
            for column in columns {
                let normal_column_clearance = self.normal_column_clearance(column);
                clearance = clearance.union(normal_column_clearance);
            }
        }

        clearance
    }

    fn normal_column_clearance(&self, column: &Column) -> Tree {
        let points = self.clearance_points(column);
        let first = column.first();
        let plane = Plane::new(
            side_point(first, Side::Left, &self.columns.key_clearance),
            first.x_axis,
        );

        prism_from_projected_points(points, &plane, 2.0 * self.columns.key_clearance.x)
    }

    fn side_column_clearance(
        &self,
        column: &Column,
        neighbor: Option<&Column>,
        side_x: SideX,
    ) -> Tree {
        let normal_offset = self.columns.key_clearance.x;
        let side_offset = self.bounds.diameter();
        let extrusion_height = 2.0 * side_offset;

        // Column clearance parameters
        let translation = column.first().translation;
        let normal = side_x.direction() * column.first().x_axis;
        let points = self.clearance_points(column);

        // Combined column and neighbor clearance
        let combined_clearance = if let Some(neighbor) = neighbor {
            let plane = Plane::new(translation - side_offset * normal, normal);
            let column_clearance = prism_from_projected_points(points, &plane, extrusion_height);

            let translation = neighbor.first().translation;
            let normal = side_x.direction() * neighbor.first().x_axis;
            let plane = Plane::new(translation - normal_offset * normal, normal);
            let points = self.clearance_points(neighbor);
            let neighbor_clearance = prism_from_projected_points(points, &plane, extrusion_height);

            column_clearance.intersection(neighbor_clearance)
        } else {
            let plane = Plane::new(translation - normal_offset * normal, normal);
            prism_from_projected_points(points, &plane, extrusion_height)
        };

        // Combined column, side and neighbor clearance
        let side_clearance = self.side_clearance(side_x);
        #[allow(clippy::if_not_else)]
        #[allow(clippy::float_cmp)]
        if column.first().x_axis.z.signum() != side_x.direction() {
            combined_clearance.intersection(side_clearance)
        } else {
            // Bound side clearance since it is combined by union and can interfere with other columns
            let bounds = self.bounds.projected_unit_vectors(normal);
            let points = [
                -bounds.y_axis,
                bounds.y_axis,
                bounds.y_axis + bounds.z_axis,
                -bounds.y_axis + bounds.z_axis,
            ];
            let plane = Plane::new(translation - normal_offset * normal, normal);
            let side_bounding_shape = prism_from_projected_points(points, &plane, extrusion_height);

            let intersection = side_clearance.intersection(side_bounding_shape);
            intersection.union(combined_clearance)
        }
    }

    fn side_clearance(&self, side_x: SideX) -> Tree {
        let column = match side_x {
            SideX::Left => self.columns.first(),
            SideX::Right => self.columns.last(),
        };

        let plane = Plane::new(self.bounds.min, DVec3::X);
        let bounds = self.bounds.projected_unit_vectors(plane.normal());

        let first = column.first();
        let last = column.last();

        let lower_corner = corner_point(first, side_x, SideY::Bottom, &self.columns.key_clearance);
        let upper_corner = corner_point(last, side_x, SideY::Top, &self.columns.key_clearance);
        let outwards_bottom_point = lower_corner - bounds.y_axis;
        let outwards_top_point = upper_corner + bounds.y_axis;
        let upwards_bottom_point = outwards_bottom_point + bounds.z_axis;
        let upwards_top_point = outwards_top_point + bounds.z_axis;

        let points: Vec<_> = column
            .windows(2)
            .filter_map(|window| self.side_point(&window[0], &window[1], side_x))
            .chain([
                upper_corner,
                outwards_top_point,
                upwards_top_point,
                upwards_bottom_point,
                outwards_bottom_point,
                lower_corner,
            ])
            .collect();

        prism_from_projected_points(points, &plane, self.bounds.size().x)
    }

    fn side_point(&self, bottom: &DAffine3, top: &DAffine3, side_x: SideX) -> Option<DVec3> {
        let outwards_direction = bottom.x_axis;

        // Get point which is more outward
        let offset = (top.translation - bottom.translation).dot(outwards_direction);
        #[allow(clippy::float_cmp)]
        let offset = if offset.signum() == side_x.direction() {
            offset
        } else {
            0.0
        };
        let point = side_point(bottom, side_x.into(), &self.columns.key_clearance)
            + offset * outwards_direction;

        let line = Line::new(point, bottom.y_axis);
        let plane = Plane::new(top.translation, top.z_axis);

        plane.intersection(&line)
    }

    fn clearance_points(&self, column: &Column) -> Vec<DVec3> {
        let first = column.first();
        let last = column.last();
        let bounds = self.bounds.projected_unit_vectors(first.x_axis);

        // All points in the center, if any
        let mut points: Vec<_> = column
            .windows(2)
            .filter_map(|window| {
                let position = window[0];
                let next_position = window[1];
                let line = Line::new(position.translation, position.y_axis);
                let plane = Plane::new(next_position.translation, next_position.z_axis);

                plane.intersection(&line)
            })
            .collect();

        // Upper and lower support points derived from the first and last entries
        let mut lower_support_points = self.support_planes.calculate_support_points(
            first,
            Side::Bottom,
            &column.column_type,
            &self.columns.key_clearance,
            &bounds,
        );
        lower_support_points.reverse();
        let upper_support_points = self.support_planes.calculate_support_points(
            last,
            Side::Top,
            &column.column_type,
            &self.columns.key_clearance,
            &bounds,
        );

        // Combine points with upper and lower support points
        points.extend(upper_support_points);
        points.extend(lower_support_points);

        points
    }
}

struct SupportPlanes {
    lower_plane: Plane,
    upper_plane: Plane,
}

impl SupportPlanes {
    fn from_columns(columns: &Columns) -> Self {
        let key_clearance = &columns.key_clearance;
        let reference_column = columns.get(1).unwrap_or_else(|| columns.first());
        let x_axis = reference_column.first().x_axis;
        let normal = x_axis.cross(DVec3::Y);

        let mut lower_points: Vec<_> = columns
            .iter()
            .filter_map(|column| match column.column_type {
                ColumnType::Normal => Some(side_point(column.first(), Side::Bottom, key_clearance)),
                ColumnType::Side => None,
            })
            .collect();

        let mut upper_points: Vec<_> = columns
            .iter()
            .filter_map(|column| match column.column_type {
                ColumnType::Normal => Some(side_point(column.last(), Side::Top, key_clearance)),
                ColumnType::Side => None,
            })
            .collect();

        let lower_plane = Self::calculate_median_plane(normal, &mut lower_points);
        let upper_plane = Self::calculate_median_plane(normal, &mut upper_points);

        Self {
            lower_plane,
            upper_plane,
        }
    }

    fn calculate_median_plane(normal: DVec3, points: &mut [DVec3]) -> Plane {
        let n = points.len() / 2;
        let (_, &mut median_point, _) = points
            .select_nth_unstable_by(n, |&position1, &position2| {
                normal.dot(position1).total_cmp(&normal.dot(position2))
            });

        Plane::new(median_point, normal)
    }

    fn calculate_support_points(
        &self,
        position: &DAffine3,
        side: Side,
        column_type: &ColumnType,
        key_clearance: &DVec2,
        bounds: &DMat3,
    ) -> Vec<DVec3> {
        const ALLOWED_DEVIATION: f64 = 1.0;

        let (sign, plane) = if matches!(side, Side::Top) {
            (1.0, &self.upper_plane)
        } else {
            (-1.0, &self.lower_plane)
        };
        let point_direction = sign * position.y_axis;
        let point = side_point(position, side, key_clearance);

        let point_is_above = plane.signed_distance_to(point) > 0.0;
        let point_direction_is_upwards = point_direction.dot(plane.normal()) > 0.0;

        let projected_point = match column_type {
            ColumnType::Normal => {
                let default = if point_is_above == point_direction_is_upwards {
                    let line = Line::new(point, position.z_axis);

                    plane.intersection(&line).unwrap()
                } else {
                    point.project_to(plane)
                };

                let line = Line::new(point, position.y_axis);
                plane.intersection(&line).map_or(default, |point| {
                    if point.abs_diff_eq(default, ALLOWED_DEVIATION) {
                        point
                    } else {
                        default
                    }
                })
            }
            ColumnType::Side => point,
        };

        let mut points = vec![point];
        if !point.abs_diff_eq(projected_point, EPSILON) {
            points.push(projected_point);
        }

        let outwards_point = projected_point + sign * bounds.y_axis;
        let upwards_point = outwards_point + bounds.z_axis;
        points.extend([outwards_point, upwards_point]);

        points
    }
}
