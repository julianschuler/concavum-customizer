use std::ops::Mul;

use glam::{dvec2, dvec3, DAffine3, DVec2, DVec3};
use hex_color::HexColor;
use opencascade::primitives::{Compound, Face, IntoShape, Shape, Surface};

use crate::model::{config::PositiveDVec2, helper::ZipNeighbors};

use super::{
    config::{Config, FingerCluster},
    helper::{Rotate, Translate},
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

        let top_connectors = plate_points.clone().flat_map(|column| {
            column
                .zip_neighbors()
                .map(|(plate, upper)| Self::top_connector(plate, upper))
        });

        let side_connectors =
            plate_points
                .clone()
                .zip_neighbors()
                .flat_map(|(column, next_column)| {
                    column
                        .zip(next_column)
                        .map(|(plate, right)| Self::side_connector(plate, right))
                });

        let diagonal_connectors =
            plate_points
                .clone()
                .zip_neighbors()
                .flat_map(|(column, next_column)| {
                    column.zip_neighbors().zip(next_column.zip_neighbors()).map(
                        |((plate, upper), (right, diagonal))| {
                            Self::diagonal_connector(plate, upper, right, diagonal)
                        },
                    )
                });

        let faces = plates
            .chain(top_connectors)
            .chain(side_connectors)
            .chain(diagonal_connectors)
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
        Surface::bezier([[plate.lt, upper.lb], [plate.rt, upper.rb]])
    }

    fn side_connector(plate: PlatePoints, right: PlatePoints) -> Surface {
        Surface::bezier([[plate.rb, plate.rt], [right.lb, right.lt]])
    }

    fn diagonal_connector(
        plate: PlatePoints,
        upper: PlatePoints,
        right: PlatePoints,
        diagonal: PlatePoints,
    ) -> Surface {
        Surface::bezier([[plate.rt, upper.rb], [right.lt, diagonal.lb]])
    }
}

impl From<KeyCluster> for Component {
    fn from(cluster: KeyCluster) -> Self {
        Component::new(cluster.shape, cluster.color)
    }
}

struct PlatePoints {
    lb: DVec3,
    lt: DVec3,
    rb: DVec3,
    rt: DVec3,
}
