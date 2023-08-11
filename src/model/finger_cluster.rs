use std::ops::Mul;

use glam::{dvec2, dvec3, DAffine3, DVec2};
use hex_color::HexColor;
use opencascade::primitives::{Compound, Edge, Face, IntoShape, Shape, Wire};

use crate::model::{
    config::{Config, FingerCluster, PositiveDVec2},
    geometry::{Line, Plane, Rotate, Translate, ZipNeighbors},
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

        let tilting = config.keyboard.tilting_angle;
        let (tilting_x, tilting_y) = (tilting.x.to_radians(), tilting.y.to_radians());
        let tilting_transform = DAffine3::from_rotation_x(tilting_x).rotate_y(tilting_y);

        let key_positions = KeyPositions::from_config(&config.finger_cluster);
        let key_positions = tilting_transform * key_positions;

        let columns = key_positions.positions.iter().map(|column| {
            let mut lines = Vec::new();

            if let (Some(first), Some(last)) = (column.first(), column.last()) {
                // First line
                lines.push(Line::new(
                    first.translation - PLATE_Y_2 * first.y_axis,
                    first.x_axis,
                ));

                // All lines in the center, if any
                column
                    .get(1)
                    .map(|next_position| {
                        let line = Line::new(first.translation, first.y_axis);
                        let plane = Plane::new(next_position.translation, next_position.z_axis);

                        line.intersection_parameter(&plane)
                    })
                    .flatten()
                    .map(|half_plate_size| {
                        for position in column.iter().skip(1) {
                            let point = Line::new(position.translation, position.y_axis)
                                .parametric_point(-half_plate_size);
                            lines.push(Line::new(point, position.x_axis));
                        }
                    });

                // Last line
                lines.push(Line::new(
                    last.translation + PLATE_Y_2 * last.y_axis,
                    last.x_axis,
                ));
            }

            lines
        });

        let column_edges = columns.map(|column| {
            let edges: Vec<_> = column
                .into_iter()
                .map(|line| {
                    Edge::segment(
                        line.parametric_point(-PLATE_X_2),
                        line.parametric_point(PLATE_X_2),
                    )
                })
                .collect();

            edges
        });

        let faces = column_edges.flat_map(|edges| {
            let faces: Vec<_> = edges
                .iter()
                .zip_neighbors()
                .map(|(edge, next_edge)| {
                    let left_edge = Edge::segment(edge.start_point(), next_edge.start_point());
                    let right_edge = Edge::segment(edge.end_point(), next_edge.end_point());

                    Face::from_wire(&Wire::from_edges([
                        edge,
                        &left_edge,
                        &right_edge,
                        next_edge,
                    ]))
                })
                .collect();

            faces
        });

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
}

impl From<KeyCluster> for Component {
    fn from(cluster: KeyCluster) -> Self {
        Component::new(cluster.shape, cluster.color)
    }
}
