use std::iter::once;

use model::matrix_pcb::{
    ClusterConnector, ColumnConnector, ColumnKeyConnectors, MatrixPcb as Model,
    NormalColumnConnector, Segment, SideColumnConnector, ThumbKeyConnectors,
    CLUSTER_CONNECTOR_ARC_RADIUS, CONNECTOR_WIDTH, FPC_PAD_OFFSET, FPC_PAD_SIZE, PAD_SIZE,
};

use crate::{position, primitives::Position, unit::IntoUnit};

/// The positions of all switches.
pub struct SwitchPositions {
    pub columns: Vec<Column>,
    pub thumb_switches: ThumbSwitches,
}

impl SwitchPositions {
    /// Creates a new set of switch positions from the given model.
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

/// The positions of a column of finger key switches.
pub struct Column {
    home_switch: Position,
    switches_below: Vec<Position>,
    switches_above: Vec<Position>,
}

impl Column {
    /// Creates a new column from the corresponding key connectors and home switch position.
    pub fn from_key_connectors(
        key_connectors: &ColumnKeyConnectors,
        home_switch: Position,
        home_row_index: usize,
    ) -> Self {
        let y_offset = -key_connectors.connector.length() - PAD_SIZE.y;
        let (offsets_below, offsets_above) = key_connectors.offsets.split_at(home_row_index);

        let mut position = home_switch;
        let switches_below = offsets_below
            .iter()
            .rev()
            .map(|&x_offset| {
                position -= position!(x_offset, y_offset, None);

                position
            })
            .collect();

        let mut position = home_switch;
        let switches_above = offsets_above
            .iter()
            .map(|&x_offset| {
                position += position!(x_offset, y_offset, None);

                position
            })
            .collect();

        Column {
            home_switch,
            switches_below,
            switches_above,
        }
    }

    /// Returns the positions of the switches in the column.
    pub fn positions(&self) -> impl Iterator<Item = &Position> {
        self.switches_below
            .iter()
            .rev()
            .chain(once(&self.home_switch))
            .chain(&self.switches_above)
    }

    /// Returns the position of the first switch in the column.
    pub fn first(&self) -> Position {
        self.switches_below
            .last()
            .copied()
            .unwrap_or(self.home_switch)
    }
}

/// The positions of the thumb key switches.
pub struct ThumbSwitches(Vec<Position>);

impl ThumbSwitches {
    /// Creates a new set of thumb keys from the corresponding key connectors and position of the first one.
    pub fn from_key_connectors(
        key_connectors: &ThumbKeyConnectors,
        first_switch: Position,
    ) -> Self {
        let switch_distance = key_connectors.connector.length() + PAD_SIZE.x;

        #[allow(clippy::cast_precision_loss)]
        let positions = (0..=key_connectors.positions.len())
            .map(|i| first_switch + position!(i as f64 * switch_distance, 0, None))
            .collect();

        Self(positions)
    }

    /// Returns the positions of the thumb keys.
    pub fn positions(&self) -> &[Position] {
        &self.0
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
