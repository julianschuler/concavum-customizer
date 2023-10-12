use glam::{dvec2, dvec3, DAffine3, DMat3, DVec2, DVec3};
use hex_color::HexColor;
use opencascade::primitives::{IntoShape, Shape, Solid, Wire};

use crate::model::{
    config::{Config, PositiveDVec2},
    geometry::{zvec, Line, Plane},
    key_positions::KeyPositions,
    Component,
};

pub struct KeyCluster {
    shape: Shape,
    color: HexColor,
    key_positions: KeyPositions,
}

impl KeyCluster {
    pub fn from_config(config: &Config) -> Self {
        const KEY_CLEARANCE: f64 = 1.0;

        let key_positions =
            KeyPositions::from_config(&config.finger_cluster).tilt(config.keyboard.tilting_angle);

        let key_distance: PositiveDVec2 = (&config.finger_cluster.key_distance).into();
        let key_clearance = dvec2(
            key_distance.x + KEY_CLEARANCE,
            key_distance.y + KEY_CLEARANCE,
        );

        let mount = Mount::from_positions(&key_positions, *config.keyboard.circumference_distance);
        let mount_height = mount.height;
        let mut shape = mount.into_shape();

        let clearances = key_positions
            .columns
            .iter()
            .map(|column| Self::column_clearance(column, &key_clearance, mount_height));

        for clearance in clearances {
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

    fn calculate_planes(
        key_positions: &KeyPositions,
        key_clearance: &DVec2,
    ) -> Option<(Plane, Plane)> {
        let columns = &key_positions.columns;
        let second_column = columns.get(2)?;
        let x_axis = first_in_column(second_column).x_axis;
        let normal = x_axis.cross(DVec3::Y);

        let mut lower_points: Vec<_> = columns
            .iter()
            .map(|column| {
                let position = last_in_column(column);

                position.translation - key_clearance.y * position.y_axis
            })
            .collect();

        let mut upper_points: Vec<_> = columns
            .iter()
            .map(|column| {
                let position = last_in_column(column);

                position.translation + key_clearance.y * position.y_axis
            })
            .collect();

        let lower_plane = Self::calculate_median_plane(normal, &mut lower_points);
        let upper_plane = Self::calculate_median_plane(normal, &mut upper_points);

        Some((lower_plane, upper_plane))
    }

    fn calculate_median_plane(normal: DVec3, points: &mut Vec<DVec3>) -> Plane {
        points.sort_unstable_by(|position1, position2| {
            normal.dot(*position1).total_cmp(&normal.dot(*position2))
        });

        let median_point = points[points.len() / 2];

        Plane::new(median_point, normal)
    }

    fn column_clearance(column: &[DAffine3], key_clearance: &DVec2, height: f64) -> Solid {
        const SIDE: f64 = 50.0;

        let first = first_in_column(column);
        let last = last_in_column(column);

        let mut lines = Vec::new();

        // First line
        lines.push(Line::new(
            first.translation - Mount::PLATE_Y_2 * first.y_axis,
            first.x_axis,
        ));

        // All lines in the center, if any
        for window in column.windows(2) {
            let position = window[0];
            let next_position = window[1];
            let line = Line::new(position.translation, position.y_axis);
            let plane = Plane::new(next_position.translation, next_position.z_axis);

            if let Some(point) = plane.intersection(&line) {
                lines.push(Line::new(point, position.x_axis));
            }
        }

        // Last line
        lines.push(Line::new(
            last.translation + Mount::PLATE_Y_2 * last.y_axis,
            last.x_axis,
        ));

        // Get polygon points by intersecting with line
        let normal = first.x_axis;
        let plane = Plane::new(first.translation - key_clearance.x / 2.0 * normal, normal);

        let mut points: Vec<_> = lines
            .into_iter()
            .map(|line| {
                plane
                    .intersection(&line)
                    .expect("line orthogonal to plane should always intersect")
            })
            .collect();

        let side_direction = canonical_base(first.x_axis).y_axis;
        let up = first.x_axis.cross(side_direction);

        let last = *last_in_column(&points) + SIDE * side_direction;
        let first = *first_in_column(&points) - SIDE * side_direction;

        points.extend([last, last + height * up, first + height * up, first]);

        let wire = Wire::from_ordered_points(points).unwrap();
        wire.to_face().extrude(key_clearance.x * normal)
    }
}

impl From<KeyCluster> for Component {
    fn from(cluster: KeyCluster) -> Self {
        Component::new(cluster.shape, cluster.color)
    }
}

struct Mount {
    shape: Solid,
    height: f64,
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

            Self::circumference_point(first_left, first_right, false, circumference_distance)
        });
        let upper_points = key_positions.columns.windows(2).map(|window| {
            let last_left = window[0].last();
            let last_right = window[1].last();

            Self::circumference_point(last_left, last_right, true, circumference_distance)
        });

        let left_bottom_corner = Self::corner_point(
            first_in_column(first_column),
            false,
            false,
            circumference_distance,
        );
        let left_top_corner = Self::corner_point(
            last_in_column(first_column),
            false,
            true,
            circumference_distance,
        );
        let right_bottom_corner = Self::corner_point(
            first_in_column(last_column),
            true,
            false,
            circumference_distance,
        );
        let right_top_corner = Self::corner_point(
            last_in_column(last_column),
            true,
            true,
            circumference_distance,
        );

        let points: Vec<_> = lower_points
            .chain([right_bottom_corner, right_top_corner])
            .chain(upper_points.rev())
            .chain([left_top_corner, left_bottom_corner])
            .map(|point| dvec3(point.x, point.y, 0.0))
            .collect();

        let height = Self::calculate_height(key_positions);
        let wire = Wire::from_ordered_points(points).unwrap();
        let shape = wire.to_face().extrude(zvec(height));

        Self { shape, height }
    }

    fn corner_point(
        position: &DAffine3,
        right: bool,
        top: bool,
        circumference_distance: f64,
    ) -> DVec3 {
        let sign_x = if right { 1.0 } else { -1.0 };
        let sign_y = if top { 1.0 } else { -1.0 };

        let canonical = canonical_base(position.x_axis);

        let corner = position.translation
            + sign_x * Self::PLATE_X_2 * position.x_axis
            + sign_y * Self::PLATE_Y_2 * position.y_axis;

        corner + circumference_distance * (sign_x * canonical.x_axis + sign_y * canonical.y_axis)
    }

    fn circumference_point(
        left: &DAffine3,
        right: &DAffine3,
        top: bool,
        circumference_distance: f64,
    ) -> DVec3 {
        let sign = if top { 1.0 } else { -1.0 };
        let left_y_canonical = canonical_base(left.x_axis).y_axis;
        let right_y_canonical = canonical_base(right.x_axis).y_axis;

        let left_target = left.translation
            + sign * (Self::PLATE_Y_2 * left.y_axis + circumference_distance * left_y_canonical);
        let right_target = right.translation
            + sign * (Self::PLATE_Y_2 * right.y_axis + circumference_distance * right_y_canonical);

        let y = sign * (left_y_canonical + right_y_canonical).normalize();

        let start = (left_target + right_target) / 2.0;

        let offset_factor = y.dot(left_target - right_target).abs() / 2.0;

        start + offset_factor * y
    }

    fn calculate_height(key_positions: &KeyPositions) -> f64 {
        key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter().map(|position| position.translation.z))
            .max_by(f64::total_cmp)
            .unwrap_or_default()
            + 20.0
    }
}

impl IntoShape for Mount {
    fn into_shape(self) -> Shape {
        self.shape.into_shape()
    }
}

fn first_in_column<T>(column: &[T]) -> &T {
    column
        .first()
        .expect("there should always be at least one row")
}

fn last_in_column<T>(column: &[T]) -> &T {
    column
        .last()
        .expect("there should always be at least one row")
}

fn canonical_base(x_axis: DVec3) -> DMat3 {
    let z_canonical = DVec3::Z;
    let y_canonical = z_canonical.cross(x_axis).normalize();
    let x_canonical = y_canonical.cross(z_canonical);

    DMat3::from_cols(x_canonical, y_canonical, z_canonical)
}
