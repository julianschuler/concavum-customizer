use glam::{dvec2, dvec3, DAffine3, DVec2, DVec3};
use hex_color::HexColor;
use opencascade::primitives::{IntoShape, JoinType, Shape, Solid, Wire};

use crate::model::{
    config::{Config, PositiveDVec2, EPSILON},
    geometry::{zvec, Line, Plane, Project},
    key_positions::{Column, ColumnType, Columns, KeyPositions},
    Component,
};

pub struct KeyCluster {
    shape: Shape,
    color: HexColor,
    key_positions: KeyPositions,
}

impl KeyCluster {
    const KEY_CLEARANCE: f64 = 1.0;

    pub fn from_config(config: &Config) -> Self {
        let key_positions =
            KeyPositions::from_config(&config.finger_cluster).tilt(config.keyboard.tilting_angle);

        let key_distance: PositiveDVec2 = (&config.finger_cluster.key_distance).into();
        let key_clearance = dvec2(
            key_distance.x + Self::KEY_CLEARANCE,
            key_distance.y + Self::KEY_CLEARANCE,
        );

        let support_planes = SupportPlanes::from_positions(&key_positions);

        let mount = Mount::from_positions(&key_positions, *config.keyboard.circumference_distance);
        let mount_size = mount.size.clone();
        let mut shape = mount.into_shape();

        for clearance in Self::column_clearances(
            &key_positions.columns,
            &key_clearance,
            &support_planes,
            &mount_size,
        ) {
            shape = shape.subtract(&clearance.into()).into();
        }

        Self {
            shape,
            color: config.colors.keyboard,
            key_positions,
        }
    }

    pub fn key_positions(&self) -> Vec<DAffine3> {
        self.key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter())
            .copied()
            .collect()
    }

    fn column_clearances(
        columns: &Columns,
        key_clearance: &DVec2,
        support_planes: &SupportPlanes,
        mount_size: &MountSize,
    ) -> Vec<Shape> {
        let first = columns.first();
        let last = columns.last();

        let mut clearances = Vec::new();

        let normal_start = match first.column_type {
            ColumnType::Normal => 0,
            ColumnType::Side => {
                let right_neighbor = columns
                    .get(1)
                    .expect("there has to be at least one normal column");
                let side_column_clearance = Self::side_column_clearance(
                    first,
                    right_neighbor,
                    true,
                    key_clearance,
                    support_planes,
                    mount_size,
                );
                clearances.push(side_column_clearance);

                2
            }
        };

        let normal_end = match last.column_type {
            ColumnType::Normal => columns.len(),
            ColumnType::Side => {
                let left_neighbor = columns
                    .get(columns.len() - 2)
                    .expect("there has to be at least one normal column");
                let side_column_clearance = Self::side_column_clearance(
                    left_neighbor,
                    last,
                    false,
                    key_clearance,
                    support_planes,
                    mount_size,
                );
                clearances.push(side_column_clearance);

                columns.len() - 2
            }
        };

        if normal_start < normal_end {
            clearances.extend(columns[normal_start..normal_end].iter().map(|column| {
                Self::normal_column_clearance(column, key_clearance, support_planes, mount_size)
                    .into()
            }));
        }

        clearances
    }

    fn normal_column_clearance(
        column: &Column,
        key_clearance: &DVec2,
        support_planes: &SupportPlanes,
        mount_size: &MountSize,
    ) -> Solid {
        let points = Self::clearance_points(column, support_planes, mount_size);
        let first = column.first();
        let normal = first.x_axis;
        let plane = Plane::new(first.translation - key_clearance.x / 2.0 * normal, normal);

        project_points_to_plane_and_extrude(points, plane, key_clearance.x)
    }

    fn side_column_clearance(
        left: &Column,
        right: &Column,
        is_left: bool,
        key_clearance: &DVec2,
        support_planes: &SupportPlanes,
        mount_size: &MountSize,
    ) -> Shape {
        let (left_offset, right_offset) = if is_left {
            (2.0 * key_clearance.x, key_clearance.x / 2.0)
        } else {
            (key_clearance.x / 2.0, 2.0 * key_clearance.x)
        };
        let extrusion_height = 4.0 * key_clearance.x;

        // Left clearance
        let first = left.first();
        let normal = first.x_axis;
        let plane = Plane::new(first.translation - left_offset * normal, normal);
        let points = Self::clearance_points(left, support_planes, mount_size);
        let left_clearance = project_points_to_plane_and_extrude(points, plane, extrusion_height);

        // Right clearance
        let first = right.first();
        let normal = first.x_axis;
        let plane = Plane::new(first.translation + right_offset * normal, normal);
        let points = Self::clearance_points(right, support_planes, mount_size);
        let right_clearance = project_points_to_plane_and_extrude(points, plane, -extrusion_height);

        left_clearance.intersect(&right_clearance).into()
    }

    fn clearance_points(
        column: &Column,
        support_planes: &SupportPlanes,
        mount_size: &MountSize,
    ) -> Vec<DVec3> {
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
        let mut lower_support_points =
            support_planes.calculate_support_points(first, false, mount_size.width);
        let upper_support_points =
            support_planes.calculate_support_points(last, true, mount_size.width);

        // Combine upper and lower support points with clearance points to polygon points
        let lower_clearance_point =
            *lower_support_points.last().unwrap() + 2.0 * mount_size.height * DVec3::Z;
        let upper_clearance_point =
            *upper_support_points.last().unwrap() + 2.0 * mount_size.height * DVec3::Z;
        points.extend(upper_support_points);
        points.extend([upper_clearance_point, lower_clearance_point]);
        lower_support_points.reverse();
        points.extend(lower_support_points);

        points
    }
}

impl From<KeyCluster> for Component {
    fn from(cluster: KeyCluster) -> Self {
        Component::new(cluster.shape, cluster.color)
    }
}

struct SupportPlanes {
    lower_plane: Plane,
    upper_plane: Plane,
}

impl SupportPlanes {
    fn from_positions(key_positions: &KeyPositions) -> Self {
        let columns = &key_positions.columns;
        let reference_column = columns.get(1).unwrap_or_else(|| columns.first());
        let x_axis = reference_column.first().x_axis;
        let normal = x_axis.cross(DVec3::Y);

        let mut lower_points: Vec<_> = columns
            .iter()
            .map(|column| {
                let position = column.first();

                position.translation - Mount::PLATE_Y_2 * position.y_axis
            })
            .collect();

        let mut upper_points: Vec<_> = columns
            .iter()
            .map(|column| {
                let position = column.last();

                position.translation + Mount::PLATE_Y_2 * position.y_axis
            })
            .collect();

        let lower_plane = Self::calculate_median_plane(normal, &mut lower_points);
        let upper_plane = Self::calculate_median_plane(normal, &mut upper_points);

        Self {
            lower_plane,
            upper_plane,
        }
    }

    fn calculate_median_plane(normal: DVec3, points: &mut Vec<DVec3>) -> Plane {
        points.sort_unstable_by(|position1, position2| {
            normal.dot(*position1).total_cmp(&normal.dot(*position2))
        });

        let median_point = points[points.len() / 2];

        Plane::new(median_point, normal)
    }

    fn calculate_support_points(
        &self,
        position: &DAffine3,
        upper: bool,
        mount_width: f64,
    ) -> Vec<DVec3> {
        let (sign, plane) = if upper {
            (1.0, &self.upper_plane)
        } else {
            (-1.0, &self.lower_plane)
        };
        let point_direction = sign * position.y_axis;
        let point = position.translation + Mount::PLATE_Y_2 * point_direction;

        let point_is_above = plane.signed_distance_to(point) > 0.0;
        let point_direction_is_upwards = point_direction.dot(plane.normal()) > 0.0;

        let projected_point = if point_is_above == point_direction_is_upwards {
            let line = Line::new(point, position.z_axis);

            plane.intersection(&line).unwrap()
        } else {
            point.project_to(plane)
        };

        let mut points = vec![point];
        if !point.abs_diff_eq(projected_point, EPSILON) {
            points.push(projected_point);
        }

        let outwards_point = projected_point + sign * mount_width * DVec3::Y;
        points.push(outwards_point);
        points
    }
}

#[derive(Clone)]
struct MountSize {
    height: f64,
    width: f64,
}

struct Mount {
    shape: Solid,
    size: MountSize,
}

impl Mount {
    const PLATE_SIZE: DVec2 = dvec2(17.0, 18.0);
    const PLATE_X_2: f64 = Self::PLATE_SIZE.x / 2.0;
    const PLATE_Y_2: f64 = Self::PLATE_SIZE.y / 2.0;

    pub fn from_positions(key_positions: &KeyPositions, circumference_distance: f64) -> Self {
        let columns = &key_positions.columns;
        let first_column = columns.first();
        let last_column = columns.last();

        let lower_points = columns.windows(2).map(|window| {
            let first_left = window[0].first();
            let first_right = window[1].first();

            Self::circumference_point(first_left, first_right, false)
        });
        let upper_points = key_positions.columns.windows(2).map(|window| {
            let last_left = window[0].last();
            let last_right = window[1].last();

            Self::circumference_point(last_left, last_right, true)
        });

        let left_bottom_corner = Self::corner_point(first_column.first(), false, false);
        let left_top_corner = Self::corner_point(first_column.last(), false, true);
        let right_bottom_corner = Self::corner_point(last_column.first(), true, false);
        let right_top_corner = Self::corner_point(last_column.last(), true, true);

        let points: Vec<_> = lower_points
            .chain([right_bottom_corner, right_top_corner])
            .chain(upper_points.rev())
            .chain([left_top_corner, left_bottom_corner])
            .map(|point| dvec3(point.x, point.y, 0.0))
            .collect();

        let size = Self::calculate_size(key_positions, &points);
        let wire = Wire::from_ordered_points(points).unwrap();
        let wire = wire.offset(circumference_distance, JoinType::Arc);
        let shape = wire.to_face().extrude(zvec(size.height));

        Self { shape, size }
    }

    fn corner_point(position: &DAffine3, right: bool, top: bool) -> DVec3 {
        let sign_x = if right { 1.0 } else { -1.0 };
        let sign_y = if top { 1.0 } else { -1.0 };

        position.translation
            + sign_x * Self::PLATE_X_2 * position.x_axis
            + sign_y * Self::PLATE_Y_2 * position.y_axis
    }

    fn circumference_point(left: &DAffine3, right: &DAffine3, top: bool) -> DVec3 {
        let sign = if top { 1.0 } else { -1.0 };

        let left_point =
            left.translation + Self::PLATE_X_2 * left.x_axis + sign * Self::PLATE_Y_2 * left.y_axis;
        let right_point = right.translation - Self::PLATE_X_2 * right.x_axis
            + sign * Self::PLATE_Y_2 * right.y_axis;

        // Get point which is more outward
        if (left_point - right_point).y.signum() == sign {
            left_point
        } else {
            right_point
        }
    }

    fn calculate_size(key_positions: &KeyPositions, points: &[DVec3]) -> MountSize {
        let height = key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter().map(|position| position.translation.z))
            .max_by(f64::total_cmp)
            .unwrap_or_default()
            + 20.0;

        let max_y = points
            .iter()
            .map(|point| point.y)
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let min_y = points
            .iter()
            .map(|point| point.y)
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let width = max_y - min_y;

        MountSize { height, width }
    }
}

impl IntoShape for Mount {
    fn into_shape(self) -> Shape {
        self.shape.into_shape()
    }
}

fn project_points_to_plane_and_extrude(
    points: impl IntoIterator<Item = DVec3>,
    plane: Plane,
    height: f64,
) -> Solid {
    let points = points.into_iter().map(|point| point.project_to(&plane));
    let wire = Wire::from_ordered_points(points).expect("wire is created from more than 2 points");

    wire.to_face().extrude(height * plane.normal())
}
