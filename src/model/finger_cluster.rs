use fidget::context::Tree;
use glam::{dvec2, dvec3, DAffine3, DVec2, DVec3, Vec3Swizzles};

use crate::{
    config::{Keyboard, PositiveDVec2, EPSILON, KEY_CLEARANCE},
    model::{
        geometry::{Line, Plane, Project},
        insert_holder::InsertHolder,
        key_positions::{Column, ColumnType, Columns},
        primitives::{Csg, RoundedCsg, SimplePolygon},
        util::{
            corner_point, prism_from_projected_points, side_point, ClusterBounds, Side, SideX,
            SideY,
        },
    },
};

pub struct FingerCluster {
    pub cluster: Tree,
    pub key_clearance: Tree,
    pub insert_holders: Tree,
    pub bounds: ClusterBounds,
}

impl FingerCluster {
    pub fn new(columns: &Columns, key_distance: &PositiveDVec2, config: &Keyboard) -> Self {
        let key_clearance = dvec2(
            key_distance.x + KEY_CLEARANCE,
            key_distance.y + KEY_CLEARANCE,
        ) / 2.0;

        let bounds = ClusterBounds::from_positions(
            columns.iter().flat_map(|column| column.iter()),
            &key_clearance,
            *config.circumference_distance,
        );

        let (outline, insert_holders) =
            Self::outline_and_insert_holders(columns, &key_clearance, config);
        let cluster_outline = outline.offset(*config.circumference_distance);
        let cluster = cluster_outline.extrude(-bounds.size.z, bounds.size.z);

        let clearance = ClearanceBuilder::new(columns, &key_clearance, &bounds).build();
        let cluster = cluster.rounded_difference(clearance, config.rounding_radius);

        let key_clearance = outline.extrude(-bounds.size.z, bounds.size.z);

        let insert_holders = insert_holders
            .into_iter()
            .map(Into::into)
            .reduce(|holders: Tree, holder| holders.union(holder))
            .expect("there is more than one insert holder for the finger cluster")
            .intersection(cluster_outline);

        Self {
            cluster,
            key_clearance,
            insert_holders,
            bounds,
        }
    }

    fn outline_and_insert_holders(
        columns: &Columns,
        key_clearance: &DVec2,
        config: &Keyboard,
    ) -> (Tree, [InsertHolder; 3]) {
        let bottom_points = columns.windows(2).map(|window| {
            let first_left = window[0].first();
            let first_right = window[1].first();

            Self::circumference_point(first_left, first_right, SideY::Bottom, key_clearance)
        });
        let top_points = columns.windows(2).map(|window| {
            let last_left = window[0].last();
            let last_right = window[1].last();

            Self::circumference_point(last_left, last_right, SideY::Top, key_clearance)
        });
        let left_points = Self::side_circumference_points(columns, SideX::Left, key_clearance);
        let right_points = Self::side_circumference_points(columns, SideX::Right, key_clearance);

        let vertices: Vec<_> = bottom_points
            .chain(right_points)
            .chain(top_points.rev())
            .chain(left_points.into_iter().rev())
            .map(Vec3Swizzles::xy)
            .collect();

        let first_index = columns.len() - 1;
        let second_index = first_index + columns.first().len();
        let third_index = second_index + columns.len();
        let insert_holders = [
            InsertHolder::from_vertices(&vertices, first_index, config),
            InsertHolder::from_vertices(&vertices, second_index, config),
            InsertHolder::from_vertices(&vertices, third_index, config),
        ];

        (SimplePolygon::new(vertices).into(), insert_holders)
    }

    fn circumference_point(
        left: &DAffine3,
        right: &DAffine3,
        side_y: SideY,
        key_clearance: &DVec2,
    ) -> DVec3 {
        let left_point = corner_point(left, SideX::Right, side_y, key_clearance);
        let right_point = corner_point(right, SideX::Left, side_y, key_clearance);

        // Get point which is more outward
        #[allow(clippy::float_cmp)]
        if (left_point - right_point).y.signum() == side_y.direction() {
            left_point
        } else {
            right_point
        }
    }

    fn side_circumference_points(
        columns: &Columns,
        side_x: SideX,
        key_clearance: &DVec2,
    ) -> Vec<DVec3> {
        let column = match side_x {
            SideX::Left => columns.first(),
            SideX::Right => columns.last(),
        };
        let first = column.first();
        let last = column.last();

        let lower_corner = corner_point(first, side_x, SideY::Bottom, key_clearance);
        let upper_corner = corner_point(last, side_x, SideY::Top, key_clearance);

        let mut points = vec![lower_corner];

        points.extend(column.windows(2).filter_map(|window| {
            Self::side_circumference_point(&window[0], &window[1], side_x, key_clearance)
        }));

        points.push(upper_corner);

        points
    }

    fn side_circumference_point(
        bottom: &DAffine3,
        top: &DAffine3,
        side_x: SideX,
        key_clearance: &DVec2,
    ) -> Option<DVec3> {
        let outwards_direction = bottom.x_axis;

        // Get point which is more outward
        let offset = (top.translation - bottom.translation).dot(outwards_direction);
        #[allow(clippy::float_cmp)]
        let offset = if offset.signum() == side_x.direction() {
            offset
        } else {
            0.0
        };
        let point = bottom.translation
            + (offset + side_x.direction() * key_clearance.x) * outwards_direction;

        let line = Line::new(point, bottom.y_axis);
        let plane = Plane::new(top.translation, top.z_axis);

        plane.intersection(&line)
    }
}

struct ClearanceBuilder<'a> {
    columns: &'a Columns,
    key_clearance: &'a DVec2,
    bounds: &'a ClusterBounds,
    support_planes: SupportPlanes,
}

impl<'a> ClearanceBuilder<'a> {
    fn new(columns: &'a Columns, key_clearance: &'a DVec2, bounds: &'a ClusterBounds) -> Self {
        let support_planes = SupportPlanes::from_columns(columns, key_clearance);

        Self {
            columns,
            key_clearance,
            bounds,
            support_planes,
        }
    }

    fn build(mut self) -> Tree {
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

    fn normal_column_clearance(&mut self, column: &Column) -> Tree {
        let points = self.clearance_points(column);
        let first = column.first();
        let plane = Plane::new(
            side_point(first, Side::Left, self.key_clearance),
            first.x_axis,
        );

        prism_from_projected_points(points, &plane, 2.0 * self.key_clearance.x)
    }

    fn side_column_clearance(
        &mut self,
        column: &Column,
        neighbor: Option<&Column>,
        side_x: SideX,
    ) -> Tree {
        let normal_offset = self.key_clearance.x;
        let side_offset = 4.0 * self.key_clearance.x;
        let extrusion_height = 8.0 * self.key_clearance.x;

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
            let plane = Plane::new(translation - normal_offset * normal, normal);
            let length = self.bounds.size.y;
            let height = self.bounds.size.z;
            let points = [
                dvec3(0.0, -length, 0.0),
                dvec3(0.0, length, 0.0),
                dvec3(0.0, length, 2.0 * height),
                dvec3(0.0, -length, 2.0 * height),
            ];
            let side_bounding_shape = prism_from_projected_points(points, &plane, extrusion_height);

            let intersection = side_clearance.intersection(side_bounding_shape);
            intersection.union(combined_clearance)
        }
    }

    fn side_clearance(&mut self, side_x: SideX) -> Tree {
        let width = self.bounds.size.x;
        let length = self.bounds.size.y;
        let height = self.bounds.size.z;

        let column = match side_x {
            SideX::Left => self.columns.first(),
            SideX::Right => self.columns.last(),
        };

        let first = column.first();
        let last = column.last();

        let lower_corner = corner_point(first, side_x, SideY::Bottom, self.key_clearance);
        let upper_corner = corner_point(last, side_x, SideY::Top, self.key_clearance);
        let outwards_bottom_point = lower_corner - length * DVec3::Y;
        let outwards_top_point = upper_corner + length * DVec3::Y;
        let upwards_bottom_point = outwards_bottom_point + 2.0 * height * DVec3::Z;
        let upwards_top_point = outwards_top_point + 2.0 * height * DVec3::Z;

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

        let plane = Plane::new(width * DVec3::NEG_X, DVec3::X);
        prism_from_projected_points(points, &plane, 2.0 * width)
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
        let point =
            side_point(bottom, side_x.into(), self.key_clearance) + offset * outwards_direction;

        let line = Line::new(point, bottom.y_axis);
        let plane = Plane::new(top.translation, top.z_axis);

        plane.intersection(&line)
    }

    fn clearance_points(&self, column: &Column) -> Vec<DVec3> {
        let first = column.first();
        let last = column.last();

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
            self.key_clearance,
            self.bounds,
        );
        lower_support_points.reverse();
        let upper_support_points = self.support_planes.calculate_support_points(
            last,
            Side::Top,
            &column.column_type,
            self.key_clearance,
            self.bounds,
        );

        // Combine upper and lower support points with clearance points to polygon points
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
    fn from_columns(columns: &Columns, key_clearance: &DVec2) -> Self {
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
        bounds: &ClusterBounds,
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

        let outwards_point = projected_point + sign * bounds.size.y * DVec3::Y;
        let upwards_point = outwards_point + 2.0 * bounds.size.y * DVec3::Z;
        points.extend([outwards_point, upwards_point]);

        points
    }
}
