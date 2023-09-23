use std::ops::Mul;

use glam::{dvec2, dvec3, DAffine3, DMat3, DVec2, DVec3};
use hex_color::HexColor;
use opencascade::primitives::{IntoShape, Shape, Solid, Wire};

use crate::model::{
    config::{Config, FingerCluster, PositiveDVec2},
    geometry::{zvec, Line, Plane, Rotate, Translate},
    key::Switch,
    Component,
};

struct KeyPositions {
    positions: Vec<Vec<DAffine3>>,
}

impl KeyPositions {
    fn from_config(config: &FingerCluster) -> Self {
        const CURVATURE_HEIGHT: f64 = Switch::TOP_HEIGHT;

        let (left_side_angle, right_side_angle) = config.side_angles;
        let key_distance: PositiveDVec2 = (&config.key_distance).into();

        let positions = config
            .columns
            .iter()
            .enumerate()
            .map(|(i, column)| {
                let (side_angle, side) = match i {
                    0 => (*left_side_angle, 1.0),
                    i if i == config.columns.len() - 1 => (*right_side_angle, -1.0),
                    _ => (0.0, 0.0),
                };
                let side_angle = side_angle.to_radians();
                let side_angle_tan = side_angle.tan();

                let (x, z_offset) = if side_angle == 0.0 {
                    (key_distance.x * i as f64, 0.0)
                } else {
                    let (sin, cos) = (side_angle.sin(), side_angle.cos());
                    let side_radius =
                        key_distance.x / 2.0 / (side_angle / 2.0).tan() + CURVATURE_HEIGHT;

                    (
                        key_distance.x * (i as f64 + side) - side * side_radius * sin,
                        side_radius * (1.0 - cos),
                    )
                };

                let translation = dvec3(x, column.offset.x, column.offset.y + z_offset);
                let column_transform =
                    DAffine3::from_rotation_y(side * side_angle).translate(translation);

                let curvature_angle = column.curvature_angle.to_radians();
                if curvature_angle == 0.0 {
                    (0..config.rows)
                        .map(|j| {
                            let y =
                                key_distance.y * (j as i16 - config.home_row_index as i16) as f64;
                            column_transform * DAffine3::from_translation(dvec3(0.0, y, 0.0))
                        })
                        .collect()
                } else {
                    let keycap_radius = key_distance.y / 2.0 / (curvature_angle / 2.0).tan();
                    let curvature_radius = keycap_radius + CURVATURE_HEIGHT;

                    (0..config.rows)
                        .map(|j| {
                            let total_angle =
                                curvature_angle * (j as i16 - config.home_row_index as i16) as f64;
                            let (sin, rcos) = (total_angle.sin(), 1.0 - total_angle.cos());

                            let x = -side
                                * side_angle_tan
                                * (keycap_radius * rcos
                                    + side_angle.signum() * key_distance.y / 2.0 * sin.abs());
                            let y = curvature_radius * sin;
                            let z = curvature_radius * rcos;

                            column_transform
                                * DAffine3::from_rotation_x(total_angle).translate(dvec3(x, y, z))
                        })
                        .collect()
                }
            })
            .collect();

        Self { positions }
    }

    pub fn tilt(self, tilting_angle: DVec2) -> Self {
        const Z_OFFSET: f64 = 12.0;

        let (tilting_x, tilting_y) = (tilting_angle.x.to_radians(), tilting_angle.y.to_radians());
        let tilting_transform = DAffine3::from_rotation_x(tilting_x).rotate_y(tilting_y);

        let tilted_positions = tilting_transform * self;

        let z_offset = Z_OFFSET
            - tilted_positions
                .positions
                .iter()
                .flat_map(|column| column.iter().map(|position| position.translation.z))
                .min_by(f64::total_cmp)
                .unwrap_or_default();

        DAffine3::from_translation(zvec(z_offset)) * tilted_positions
    }
}

impl Mul<KeyPositions> for DAffine3 {
    type Output = KeyPositions;

    fn mul(self, cluster: KeyPositions) -> Self::Output {
        let positions = cluster
            .positions
            .into_iter()
            .map(|column| column.into_iter().map(|position| self * position).collect())
            .collect();

        KeyPositions { positions }
    }
}

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
            .positions
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
            .positions
            .iter()
            .flatten()
            .copied()
            .collect()
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
        let lower_points = key_positions.positions.windows(2).map(|window| {
            let first_left = first_in_column(&window[0]);
            let first_right = first_in_column(&window[1]);

            Self::circumference_point(first_left, first_right, false, circumference_distance)
        });
        let upper_points = key_positions.positions.windows(2).map(|window| {
            let last_left = last_in_column(&window[0]);
            let last_right = last_in_column(&window[1]);

            Self::circumference_point(last_left, last_right, true, circumference_distance)
        });

        let first_column = key_positions.positions.first().unwrap();
        let last_column = key_positions.positions.last().unwrap();

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
            .positions
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
