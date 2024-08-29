mod segments;

use glam::{dvec2, DAffine3, DVec2, DVec3};
use segments::{Line, UpwardsArc};

use crate::{
    geometry::{Line as GeometricLine, Plane},
    key_positions::{Column, KeyPositions},
    util::SideY,
};

pub use segments::Segment;

/// The size of the PCB pads underneath each key.
pub const PAD_SIZE: DVec2 = dvec2(13.0, 14.0);
/// The thickness of the matrix PCB.
pub const THICKNESS: f64 = 0.6;
/// The width of the connectors between keys and columns.
pub const CONNECTOR_WIDTH: f64 = 2.0;

const SWITCH_HEIGHT: f64 = 5.0;

/// A PCB connecting the keys to each other in a matrix.
pub struct MatrixPcb {
    /// The key connectors between keys in the columns.
    pub key_connectors: Vec<KeyConnectors>,
}

impl MatrixPcb {
    /// Creates a new matrix PCB from the given key positions.
    pub fn from_positions(positions: &KeyPositions) -> Self {
        let key_connectors = positions
            .columns
            .iter()
            .map(KeyConnectors::from_column)
            .collect();

        Self { key_connectors }
    }
}

/// A connector between keys.
pub enum KeyConnector {
    /// An arc.
    Arc(UpwardsArc),
    /// A line.
    Line(Line),
}

impl KeyConnector {
    /// Creates a new key connector from the given column.
    #[must_use]
    fn from_column(column: &Column) -> Self {
        if let Some(next_position) = column.get(1) {
            let position = column.first();
            let start_point = key_connector_point(*position, SideY::Top);
            let end_point = key_connector_point(*next_position, SideY::Bottom);

            let plane = Plane::new(start_point, position.y_axis);
            let line = GeometricLine::new(end_point, next_position.z_axis);

            let angle = position
                .z_axis
                .dot(next_position.z_axis)
                .clamp(-1.0, 1.0)
                .acos();

            if angle == 0.0 {
                let length = start_point.distance(end_point);

                Self::Line(Line::new(length))
            } else {
                let intersection = plane.intersection(&line).unwrap_or_default();
                let radius = (intersection - start_point).dot(position.z_axis);

                Self::Arc(UpwardsArc::new(radius, angle))
            }
        } else {
            Self::Line(Line::new(1.0))
        }
    }
}

impl Segment for KeyConnector {
    fn positions(&self) -> Vec<DAffine3> {
        match self {
            KeyConnector::Arc(arc) => arc.positions(),
            KeyConnector::Line(line) => line.positions(),
        }
    }

    fn length(&self) -> f64 {
        match self {
            KeyConnector::Arc(arc) => arc.length(),
            KeyConnector::Line(line) => line.length(),
        }
    }
}

/// The connectors between keys in a column.
pub struct KeyConnectors {
    /// The connector itself.
    pub connector: KeyConnector,
    /// The positions of the connector.
    pub positions: Vec<DAffine3>,
}

impl KeyConnectors {
    /// Creates new key connectors for a given column.
    #[must_use]
    fn from_column(column: &Column) -> Self {
        let positions = column
            .windows(2)
            .flat_map(|window| {
                let position = window[0];
                let next_position = window[1];

                let transformed_next_key = position
                    .inverse()
                    .transform_point3(next_position.translation);

                let left_x_offset =
                    transformed_next_key.x.max(0.0) - (PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0;
                let right_x_offset =
                    transformed_next_key.x.min(0.0) + (PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0;

                let start_point = key_connector_point(position, SideY::Top);
                let left_position = DAffine3 {
                    matrix3: position.matrix3,
                    translation: start_point + left_x_offset * position.x_axis,
                };
                let right_position = DAffine3 {
                    matrix3: position.matrix3,
                    translation: start_point + right_x_offset * position.x_axis,
                };

                [left_position, right_position]
            })
            .collect();
        let connector = KeyConnector::from_column(column);

        Self {
            connector,
            positions,
        }
    }
}

fn key_connector_point(position: DAffine3, side: SideY) -> DVec3 {
    position.translation + side.direction() * PAD_SIZE.y / 2.0 * position.y_axis
        - (SWITCH_HEIGHT + THICKNESS / 2.0) * position.z_axis
}
