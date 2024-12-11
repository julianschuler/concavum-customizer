mod column;
mod thumb_switches;

use std::iter::once;

use model::matrix_pcb::{
    ClusterConnector, ColumnConnector, MatrixPcb as Model, NormalColumnConnector, Segment,
    SideColumnConnector, CLUSTER_CONNECTOR_ARC_RADIUS, CONNECTOR_WIDTH, FPC_PAD_OFFSET,
    FPC_PAD_SIZE, PAD_SIZE,
};

use crate::{position, primitives::Position, unit::IntoUnit};

pub use column::Column;
pub use thumb_switches::ThumbSwitches;

/// All features of the matrix PCB.
pub struct Features {
    pub columns: Vec<Column>,
    pub thumb_switches: ThumbSwitches,
}

impl Features {
    /// Calculates all features of the matrix PCB from the model.
    pub fn from_model(
        model: &Model,
        home_row_index: usize,
        cluster_connector_index: usize,
    ) -> Self {
        let mut position = position!(100, 100, None);

        let columns: Vec<_> = once(position)
            .chain(model.column_connectors.iter().map(|column_connector| {
                position += match column_connector {
                    ColumnConnector::Normal(NormalColumnConnector {
                        bezier_curve,
                        arc_radius,
                        left_arc_side,
                    }) => {
                        let connector_length = left_arc_side.direction()
                            * (2.0 * arc_radius + bezier_curve.length() - PAD_SIZE.y
                                + CONNECTOR_WIDTH);

                        position!(2.0 * arc_radius + PAD_SIZE.x, connector_length, None)
                    }
                    ColumnConnector::Side(SideColumnConnector { connector, .. }) => {
                        position!(connector.length() + PAD_SIZE.x, 0, None)
                    }
                };

                position
            }))
            .zip(&model.column_key_connectors)
            .map(|(position, column_key_connectors)| {
                Column::from_key_connectors(column_key_connectors, position, home_row_index)
            })
            .collect();

        let anchor_position = columns[cluster_connector_index].first();
        let first_thumb_switch =
            first_thumb_switch_position(anchor_position, &model.cluster_connector);
        let thumb_switches =
            ThumbSwitches::from_key_connectors(&model.thumb_key_connectors, first_thumb_switch);

        Self {
            columns,
            thumb_switches,
        }
    }
}

/// Calculates the position of the first thumb switch.
fn first_thumb_switch_position(
    anchor_position: Position,
    cluster_connector: &ClusterConnector,
) -> Position {
    anchor_position
        + position!(
            -CLUSTER_CONNECTOR_ARC_RADIUS,
            FPC_PAD_OFFSET + FPC_PAD_SIZE.y / 2.0,
            Some(-cluster_connector.finger_cluster_arc_angle.rad())
        )
        + position!(
            2.0 * CLUSTER_CONNECTOR_ARC_RADIUS,
            cluster_connector.bezier_curve.length(),
            Some(cluster_connector.thumb_cluster_arc_angle.rad())
        )
        + position!(-CLUSTER_CONNECTOR_ARC_RADIUS, PAD_SIZE.y / 2.0, None)
}
