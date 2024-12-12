mod column;
mod thumb_switches;

use std::iter::once;

use model::matrix_pcb::{MatrixPcb as Model, FPC_PAD_OFFSET, FPC_PAD_SIZE};

use crate::{
    matrix_pcb::{connector::Connector, ORIGIN_POSITION},
    position,
    unit::IntoUnit,
};

pub use column::Column;
pub use thumb_switches::ThumbSwitches;

/// All features of the matrix PCB.
pub struct Features {
    pub columns: Vec<Column>,
    pub thumb_switches: ThumbSwitches,
    pub column_connectors: Vec<Connector>,
    pub cluster_connector: Connector,
}

impl Features {
    /// Calculates all features of the matrix PCB from the model.
    pub fn from_model(
        model: &Model,
        home_row_index: usize,
        cluster_connector_index: usize,
    ) -> Self {
        let mut switch_position = ORIGIN_POSITION;

        let column_connectors: Vec<_> = model
            .column_connectors
            .iter()
            .map(|column_connector| {
                let connector = Connector::from_column_connector(column_connector, switch_position);
                switch_position = connector.end_switch_position();
                connector
            })
            .collect();

        let columns: Vec<_> = once(ORIGIN_POSITION)
            .chain(column_connectors.iter().map(Connector::end_switch_position))
            .zip(&model.column_key_connectors)
            .map(|(position, column_key_connectors)| {
                Column::from_key_connectors(column_key_connectors, position, home_row_index)
            })
            .collect();

        let cluster_connector_start = columns[cluster_connector_index].first()
            + position!(0, FPC_PAD_OFFSET + FPC_PAD_SIZE.y / 2.0, Some(-90.deg()));
        let cluster_connector =
            Connector::from_cluster_connector(&model.cluster_connector, cluster_connector_start);

        let thumb_switches = ThumbSwitches::from_key_connectors(
            &model.thumb_key_connectors,
            cluster_connector.end_switch_position(),
        );

        Self {
            columns,
            thumb_switches,
            column_connectors,
            cluster_connector,
        }
    }
}
