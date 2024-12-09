mod cluster_connector;
mod connectors;
mod segments;

use glam::{dvec2, DAffine3, DVec2, DVec3};

use crate::key_positions::{ColumnType, KeyPositions};

pub use cluster_connector::ClusterConnector;
pub use connectors::{ColumnConnector, ColumnKeyConnectors, ThumbKeyConnectors};
pub use segments::Segment;

/// The thickness of the matrix PCB.
pub const THICKNESS: f64 = 0.6;
/// The size of the PCB pads underneath each key.
pub const PAD_SIZE: DVec2 = dvec2(13.0, 14.0);
/// The size of the PCB pad below the FPC connector.
pub const FPC_PAD_SIZE: DVec2 = dvec2(19.0, 4.0);
/// The offset of the FPC pad to the center of the key.
pub const FPC_PAD_OFFSET: f64 = 7.9;
/// The width of the connectors between keys.
pub const CONNECTOR_WIDTH: f64 = 2.0;
/// The diameter of the routing bit.
pub const ROUTER_BIT_DIAMETER: f64 = 2.0;
/// The radius of the arcs of the cluster connector.
pub const CLUSTER_CONNECTOR_ARC_RADIUS: f64 = CONNECTOR_WIDTH / 2.0 + ROUTER_BIT_DIAMETER;

const SWITCH_HEIGHT: f64 = 5.0;

/// A PCB connecting the keys to each other in a matrix.
pub struct MatrixPcb {
    /// The key connectors between keys in the columns.
    pub column_key_connectors: Vec<ColumnKeyConnectors>,
    /// The key connectors between keys in the thumb cluster.
    pub thumb_key_connectors: ThumbKeyConnectors,
    /// The connectors between neighboring columns.
    pub column_connectors: Vec<ColumnConnector>,
    /// The connector between finger and thumb cluster,
    pub cluster_connector: ClusterConnector,
    /// The position of the FPC connector pad,
    pub fpc_pad_position: DAffine3,
}

impl MatrixPcb {
    /// Creates a new matrix PCB from the given key positions.
    #[allow(clippy::missing_panics_doc)]
    pub fn from_positions(positions: &KeyPositions) -> Self {
        let columns = &positions.columns;
        #[allow(clippy::cast_sign_loss)]
        let home_row_index = columns.home_row_index as usize;

        let column_key_connectors = columns
            .iter()
            .map(ColumnKeyConnectors::from_column)
            .collect();
        let thumb_key_connectors = ThumbKeyConnectors::from_thumb_keys(&positions.thumb_keys);
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
            finger_anchor_key,
            positions.thumb_keys.first(),
        );

        let fpc_pad_position = DAffine3 {
            matrix3: finger_anchor_key.matrix3,
            translation: pad_center(finger_anchor_key)
                - FPC_PAD_OFFSET * finger_anchor_key.matrix3.y_axis,
        };

        Self {
            column_key_connectors,
            thumb_key_connectors,
            column_connectors,
            cluster_connector,
            fpc_pad_position,
        }
    }
}

/// Returns the center point of a PCB pad for the given key position.
fn pad_center(position: DAffine3) -> DVec3 {
    position.translation - (SWITCH_HEIGHT + THICKNESS / 2.0) * position.z_axis
}
