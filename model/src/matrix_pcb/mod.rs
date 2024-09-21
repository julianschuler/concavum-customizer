mod cluster_connector;
mod connectors;
mod segments;

use std::iter::once;

use glam::{dvec2, DVec2};

use crate::key_positions::{ColumnType, KeyPositions};

pub use cluster_connector::ClusterConnector;
pub use connectors::{ColumnConnector, KeyConnectors};
pub use segments::Segment;

/// The size of the PCB pads underneath each key.
pub const PAD_SIZE: DVec2 = dvec2(13.0, 14.0);
/// The thickness of the matrix PCB.
pub const THICKNESS: f64 = 0.6;
/// The width of the connectors between keys and columns.
pub const CONNECTOR_WIDTH: f64 = 2.0;
/// The diameter of the routing bit.
pub const ROUTER_BIT_DIAMETER: f64 = 2.0;
/// The radius of the arcs of the cluster connector.
pub const CLUSTER_CONNECTOR_ARC_RADIUS: f64 = CONNECTOR_WIDTH / 2.0 + ROUTER_BIT_DIAMETER;

const SWITCH_HEIGHT: f64 = 5.0;

/// A PCB connecting the keys to each other in a matrix.
pub struct MatrixPcb {
    /// The key connectors between keys in the columns.
    pub key_connectors: Vec<KeyConnectors>,
    /// The normal column connectors.
    pub column_connectors: Vec<ColumnConnector>,
    /// The connector between finger and thumb cluster,
    pub cluster_connector: ClusterConnector,
}

impl MatrixPcb {
    /// Creates a new matrix PCB from the given key positions.
    #[allow(clippy::missing_panics_doc)]
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

        let finger_anchor_key = positions
            .columns
            .iter()
            .find(|column| matches!(column.column_type, ColumnType::Normal))
            .expect("there is always at least one normal column")
            .first();
        let cluster_connector = ClusterConnector::from_anchor_key_positions(
            *finger_anchor_key,
            *positions.thumb_keys.first(),
        );

        Self {
            key_connectors,
            column_connectors,
            cluster_connector,
        }
    }
}
