use std::ops::Mul;

use glam::{dvec2, dvec3, DAffine3, DVec2, DVec3};
use hex_color::HexColor;
use opencascade::primitives::{Compound, Face, IntoShape, Shape, Surface};

use crate::model::{
    config::{Config, FingerCluster, PositiveDVec2},
    helper::{ProjectOntoPlane, Rotate, Translate, ZipNeighbors},
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
        const PLATE_SIZE: DVec2 = dvec2(17.0, 18.0);
        const PLATE_X_2: f64 = PLATE_SIZE.x / 2.0;
        const PLATE_Y_2: f64 = PLATE_SIZE.y / 2.0;
        const PLATE_POINT_LB: DVec3 = dvec3(-PLATE_X_2, -PLATE_Y_2, 0.0);
        const PLATE_POINT_LT: DVec3 = dvec3(-PLATE_X_2, PLATE_Y_2, 0.0);
        const PLATE_POINT_RB: DVec3 = dvec3(PLATE_X_2, -PLATE_Y_2, 0.0);
        const PLATE_POINT_RT: DVec3 = dvec3(PLATE_X_2, PLATE_Y_2, 0.0);

        let tilting = config.keyboard.tilting_angle;
        let (tilting_x, tilting_y) = (tilting.x.to_radians(), tilting.y.to_radians());
        let tilting_transform = DAffine3::from_rotation_x(tilting_x).rotate_y(tilting_y);

        let key_positions = KeyPositions::from_config(&config.finger_cluster);
        let key_positions = tilting_transform * key_positions;

        // Calculate points defining the plates beneath the keys first
        let plate_points = key_positions.positions.iter().map(|column| {
            column.iter().map(|position| {
                let lb = position.transform_point3(PLATE_POINT_LB);
                let lt = position.transform_point3(PLATE_POINT_LT);
                let rb = position.transform_point3(PLATE_POINT_RB);
                let rt = position.transform_point3(PLATE_POINT_RT);

                PlatePoints { lb, lt, rb, rt }
            })
        });

        let plates = plate_points
            .clone()
            .flat_map(|column| column.map(Self::plate));

        let vertical_connectors = plate_points.clone().flat_map(|column| {
            column
                .zip_neighbors()
                .map(|(plate, upper)| Self::top_connector(plate, upper))
        });

        let column_points = key_positions.positions.iter().map(|column| {
            column.iter().flat_map(|position| {
                let matrix = position.matrix3;
                let tangent = matrix.x_axis;
                let up = matrix.y_axis;
                let normal = matrix.z_axis;

                [
                    ColumnPoint {
                        left: position.transform_point3(PLATE_POINT_LB),
                        right: position.transform_point3(PLATE_POINT_RB),
                        up,
                        tangent,
                        normal,
                    },
                    ColumnPoint {
                        left: position.transform_point3(PLATE_POINT_LT),
                        right: position.transform_point3(PLATE_POINT_RT),
                        up,
                        tangent,
                        normal,
                    },
                ]
            })
        });

        let column_connectors =
            column_points
                .zip_neighbors()
                .flat_map(|(left_column, right_column)| {
                    let left_column: Vec<_> = left_column.collect();
                    let right_column: Vec<_> = right_column.collect();

                    let left_column_length = left_column.len() - 1;
                    let right_column_length = right_column.len() - 1;

                    let mut previous_points =
                        Self::calculate_next_points(&left_column[0], &right_column[0]);
                    let (mut left_index, mut right_index) = (0, 0);
                    let mut left = &left_column[left_index];
                    let mut right = &right_column[right_index];
                    let mut surfaces = Vec::new();

                    while left_index < left_column_length || right_index < right_column_length {
                        let up = left.up + right.up;

                        let next_left_index = (left_index + 1).min(left_column_length);
                        let next_right_index = (right_index + 1).min(right_column_length);

                        (left_index, right_index) = if next_left_index != left_index
                            && (left_column[next_left_index].right - right.left).dot(up) <= 0.0
                        {
                            (next_left_index, right_index)
                        } else if next_right_index != right_index
                            && (right_column[next_right_index].left - left.right).dot(up) <= 0.0
                        {
                            (left_index, next_right_index)
                        } else {
                            (next_left_index, next_right_index)
                        };

                        left = &left_column[left_index];
                        right = &right_column[right_index];

                        let next_points = Self::calculate_next_points(left, right);
                        surfaces.push(Surface::bezier([previous_points, next_points]));
                        previous_points = next_points;
                    }

                    surfaces
                });

        // Finally, combine all faces
        let faces = plates
            .chain(vertical_connectors)
            .chain(column_connectors)
            .map(|surface| Face::from_surface(&surface));

        let shape = Compound::from_shapes(faces.into_iter().map(|face| face.into_shape())).into();

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

    fn plate(plate: PlatePoints) -> Surface {
        Surface::bezier([[plate.lb, plate.lt], [plate.rb, plate.rt]])
    }

    fn top_connector(plate: PlatePoints, upper: PlatePoints) -> Surface {
        let (lb, lt, rb, rt) = (plate.lt, upper.lb, plate.rt, upper.rb);
        let normal = (lb - rb + lt - rt).normalize();
        let tangent_lb = (lb - plate.lb).normalize();
        let tangent_lt = (lt - upper.lt).normalize();
        let tangent_rb = (rb - plate.rb).normalize();
        let tangent_rt = (rt - upper.rt).normalize();

        let distance_l =
            Self::calculate_control_point_distance(lb, tangent_lb, lt, tangent_lt, normal);
        let distance_r =
            Self::calculate_control_point_distance(rb, tangent_rb, rt, tangent_rt, normal);

        Surface::bezier([
            [
                lb,
                lb + distance_l * tangent_lb,
                lt + distance_l * tangent_lt,
                lt,
            ],
            [
                rb,
                rb + distance_r * tangent_rb,
                rt + distance_r * tangent_rt,
                rt,
            ],
        ])
    }

    fn calculate_next_points(left: &ColumnPoint, right: &ColumnPoint) -> [DVec3; 4] {
        const MINIMUM_OFFSET_HEIGHT: f64 = 1.5;
        const DISTANCE_OFFSET_FACTOR: f64 = 0.2;

        let up = left.up + right.up;
        let normal = (left.normal + right.normal).normalize();
        let left_tangent = left.tangent;
        let right_tangent = -right.tangent;
        let left = left.right;
        let right = right.left;

        let distance =
            Self::calculate_control_point_distance(left, left_tangent, right, right_tangent, up);
        let distance_offset = match (left - right).dot(normal) {
            f if f.abs() >= MINIMUM_OFFSET_HEIGHT => f.signum() * DISTANCE_OFFSET_FACTOR * distance,
            _ => 0.0,
        };

        [
            left,
            left + (distance - distance_offset) * left_tangent,
            right + (distance + distance_offset) * right_tangent,
            right,
        ]
    }

    fn calculate_control_point_distance(
        p1: DVec3,
        tangent1: DVec3,
        p2: DVec3,
        tangent2: DVec3,
        normal: DVec3,
    ) -> f64 {
        let dp = (p1 - p2).project_onto_plane(normal);
        let dt = (tangent1 - tangent2).project_onto_plane(normal);
        let dot_dp_dt = dp.dot(dt);
        let norm_dp = dp.length_squared();
        let r_norm_dt = 1.0 - dt.length_squared();

        let sqrt = (dot_dp_dt * dot_dp_dt + norm_dp * r_norm_dt).sqrt();

        let candidate_equal_length1 = (dot_dp_dt + sqrt) / r_norm_dt;
        let candidate_equal_length2 = (dot_dp_dt - sqrt) / r_norm_dt;
        let candidate_perpendicular1 = -2.0 * dp.dot(tangent1) / dt.dot(tangent1);
        let candidate_perpendicular2 = -2.0 * dp.dot(tangent2) / dt.dot(tangent2);

        [
            candidate_equal_length1,
            candidate_equal_length2,
            candidate_perpendicular1,
            candidate_perpendicular2,
        ]
        .into_iter()
        .filter(|&c| c > 0.0)
        .min_by(f64::total_cmp)
        .unwrap_or(1.0)
    }
}

impl From<KeyCluster> for Component {
    fn from(cluster: KeyCluster) -> Self {
        Component::new(cluster.shape, cluster.color)
    }
}

struct ColumnPoint {
    left: DVec3,
    right: DVec3,
    up: DVec3,
    tangent: DVec3,
    normal: DVec3,
}

struct PlatePoints {
    lb: DVec3,
    lt: DVec3,
    rb: DVec3,
    rt: DVec3,
}
