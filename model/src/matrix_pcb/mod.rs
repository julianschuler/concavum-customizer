mod connectors;
mod segments;

use std::iter::once;

use glam::{dvec2, DVec2};

use crate::key_positions::KeyPositions;

pub use connectors::{ColumnConnector, KeyConnectors};
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
    /// The normal column connectors.
    pub column_connectors: Vec<ColumnConnector>,
}

impl MatrixPcb {
    /// Creates a new matrix PCB from the given key positions.
    pub fn from_positions(positions: &KeyPositions) -> Self {
        let columns = &positions.columns;
        #[allow(clippy::cast_sign_loss)]
        let home_row_index = columns.home_row_index as usize;

        let key_connectors = columns
            .iter()
            .map(KeyConnectors::from_column)
            .chain(once(KeyConnectors::from_thumb_keys(&positions.thumb_keys)))
            .collect();
        let column_connectors = columns
            .windows(2)
            .map(|window| ColumnConnector::from_columns(&window[0], &window[1], home_row_index))
            .collect();

        Self {
            key_connectors,
            column_connectors,
        }
    }
}
