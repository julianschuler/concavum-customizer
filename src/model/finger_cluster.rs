use glam::{dvec3, vec2, vec3a, Affine3A, Vec2, Vec3A};
use opencascade::primitives::{IntoShape, JoinType, Shape, Solid, Wire};

use crate::model::{
    config::{PositiveDVec2, EPSILON, KEY_CLEARANCE},
    geometry::{Line, Plane, Project},
    key_positions::{Column, ColumnType, Columns},
    util::{
        corner_point, project_points_to_plane_and_extrude, side_point, wire_from_points, MountSize,
        Side, SideX, SideY,
    },
};

pub struct FingerCluster {
    pub mount: Shape,
    pub key_clearance: Shape,
}

impl FingerCluster {
    pub fn new(
        columns: &Columns,
        key_distance: &PositiveDVec2,
        circumference_distance: f32,
    ) -> Self {
        let key_clearance = vec2(
            key_distance.x + KEY_CLEARANCE,
            key_distance.y + KEY_CLEARANCE,
        ) / 2.0;

        let size = MountSize::from_positions(
            columns.iter().flat_map(|column| column.iter()),
            &key_clearance,
            circumference_distance,
        );

        let mount_outline = Self::mount_outline(columns, &key_clearance);
        let mount_clearance = ClearanceBuilder::new(columns, &key_clearance, &size).build();
        let mount = mount_outline
            .offset(circumference_distance as f64, JoinType::Arc)
            .to_face()
            .extrude(dvec3(0.0, 0.0, size.height as f64))
            .into_shape()
            .subtract(&mount_clearance)
            .into();

        let key_clearance = mount_outline
            .to_face()
            .extrude(dvec3(0.0, 0.0, size.height as f64))
            .into();

        Self {
            mount,
            key_clearance,
        }
    }

    fn mount_outline(columns: &Columns, key_clearance: &Vec2) -> Wire {
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

        let points: Vec<_> = bottom_points
            .chain(right_points)
            .chain(top_points.rev())
            .chain(left_points.into_iter().rev())
            .map(|point| vec3a(point.x, point.y, 0.0))
            .collect();

        wire_from_points(points, &Plane::new(Vec3A::ZERO, Vec3A::Z))
    }

    fn circumference_point(
        left: &Affine3A,
        right: &Affine3A,
        side_y: SideY,
        key_clearance: &Vec2,
    ) -> Vec3A {
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
        key_clearance: &Vec2,
    ) -> Vec<Vec3A> {
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
        bottom: &Affine3A,
        top: &Affine3A,
        side_x: SideX,
        key_clearance: &Vec2,
    ) -> Option<Vec3A> {
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
    key_clearance: &'a Vec2,
    mount_size: &'a MountSize,
    support_planes: SupportPlanes,
}

impl<'a> ClearanceBuilder<'a> {
    fn new(columns: &'a Columns, key_clearance: &'a Vec2, mount_size: &'a MountSize) -> Self {
        let support_planes = SupportPlanes::from_columns(columns, key_clearance);

        Self {
            columns,
            key_clearance,
            mount_size,
            support_planes,
        }
    }

    fn build(self) -> Shape {
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

        let mut clearance = left_clearance.union(&right_clearance);

        if let Some(columns) = columns.get(1..columns.len() - 1) {
            for column in columns {
                clearance = clearance.union(&self.normal_column_clearance(column).into());
            }
        }

        clearance.into()
    }

    fn normal_column_clearance(&self, column: &Column) -> Solid {
        let points = self.clearance_points(column);
        let first = column.first();
        let plane = Plane::new(
            side_point(first, Side::Left, self.key_clearance),
            first.x_axis,
        );

        project_points_to_plane_and_extrude(points, &plane, 2.0 * self.key_clearance.x)
    }

    fn side_column_clearance(
        &self,
        column: &Column,
        neighbor: Option<&Column>,
        side_x: SideX,
    ) -> Shape {
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
            let column_clearance =
                project_points_to_plane_and_extrude(points, &plane, extrusion_height);

            let translation = neighbor.first().translation;
            let normal = side_x.direction() * neighbor.first().x_axis;
            let plane = Plane::new(translation - normal_offset * normal, normal);
            let points = self.clearance_points(neighbor);
            let neighbor_clearance =
                project_points_to_plane_and_extrude(points, &plane, extrusion_height);

            column_clearance.intersect(&neighbor_clearance).into_shape()
        } else {
            let plane = Plane::new(translation - normal_offset * normal, normal);
            let column_clearance =
                project_points_to_plane_and_extrude(points, &plane, extrusion_height);

            column_clearance.into_shape()
        };

        // Combined column, side and neighbor clearance
        let side_clearance = self.side_clearance(side_x);
        #[allow(clippy::if_not_else)]
        #[allow(clippy::float_cmp)]
        if column.first().x_axis.z.signum() != side_x.direction() {
            combined_clearance.intersect(&side_clearance).into()
        } else {
            // Bound side clearance since it is combined by union and can interfere with other columns
            let plane = Plane::new(translation - normal_offset * normal, normal);
            let points = [
                vec3a(0.0, -self.mount_size.length, 0.0),
                vec3a(0.0, self.mount_size.length, 0.0),
                vec3a(0.0, self.mount_size.length, 2.0 * self.mount_size.height),
                vec3a(0.0, -self.mount_size.length, 2.0 * self.mount_size.height),
            ];
            let side_bounding_shape =
                project_points_to_plane_and_extrude(points, &plane, extrusion_height);

            side_clearance
                .intersect(&side_bounding_shape.into())
                .union(&combined_clearance)
                .into()
        }
    }

    fn side_clearance(&self, side_x: SideX) -> Shape {
        let column = match side_x {
            SideX::Left => self.columns.first(),
            SideX::Right => self.columns.last(),
        };

        let first = column.first();
        let last = column.last();

        let lower_corner = corner_point(first, side_x, SideY::Bottom, self.key_clearance);
        let upper_corner = corner_point(last, side_x, SideY::Top, self.key_clearance);
        let outwards_bottom_point = lower_corner - self.mount_size.length * Vec3A::Y;
        let outwards_top_point = upper_corner + self.mount_size.length * Vec3A::Y;
        let upwards_bottom_point = outwards_bottom_point + 2.0 * self.mount_size.height * Vec3A::Z;
        let upwards_top_point = outwards_top_point + 2.0 * self.mount_size.height * Vec3A::Z;

        let points = column
            .windows(2)
            .filter_map(|window| self.side_point(&window[0], &window[1], side_x))
            .chain([
                upper_corner,
                outwards_top_point,
                upwards_top_point,
                upwards_bottom_point,
                outwards_bottom_point,
                lower_corner,
            ]);

        let plane = Plane::new(self.mount_size.width * Vec3A::NEG_X, Vec3A::X);
        project_points_to_plane_and_extrude(points, &plane, 2.0 * self.mount_size.width).into()
    }

    fn side_point(&self, bottom: &Affine3A, top: &Affine3A, side_x: SideX) -> Option<Vec3A> {
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

    fn clearance_points(&self, column: &Column) -> Vec<Vec3A> {
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
            self.mount_size,
        );
        lower_support_points.reverse();
        let upper_support_points = self.support_planes.calculate_support_points(
            last,
            Side::Top,
            &column.column_type,
            self.key_clearance,
            self.mount_size,
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
    fn from_columns(columns: &Columns, key_clearance: &Vec2) -> Self {
        let reference_column = columns.get(1).unwrap_or_else(|| columns.first());
        let x_axis = reference_column.first().x_axis;
        let normal = x_axis.cross(Vec3A::Y);

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

    fn calculate_median_plane(normal: Vec3A, points: &mut Vec<Vec3A>) -> Plane {
        let n = points.len() / 2;
        let (_, &mut median_point, _) = points
            .select_nth_unstable_by(n, |&position1, &position2| {
                normal.dot(position1).total_cmp(&normal.dot(position2))
            });

        Plane::new(median_point, normal)
    }

    fn calculate_support_points(
        &self,
        position: &Affine3A,
        side: Side,
        column_type: &ColumnType,
        key_clearance: &Vec2,
        mount_size: &MountSize,
    ) -> Vec<Vec3A> {
        const ALLOWED_DEVIATION: f32 = 1.0;

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

        let outwards_point = projected_point + sign * mount_size.length * Vec3A::Y;
        let upwards_point = outwards_point + 2.0 * mount_size.height * Vec3A::Z;
        points.extend([outwards_point, upwards_point]);

        points
    }
}
