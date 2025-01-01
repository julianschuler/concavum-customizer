mod column;
mod ffc_connector;
mod thumb_switches;

use std::iter::once;

use model::matrix_pcb::MatrixPcb as Model;

use crate::matrix_pcb::{connector::Connector, ORIGIN_POSITION};

pub use column::Column;
pub use ffc_connector::FfcConnector;
pub use thumb_switches::ThumbSwitches;

/// All features of the matrix PCB.
pub struct Features {
    /// The columns in the finger cluster.
    pub columns: Vec<Column>,
    /// The switches in the thumb cluster.
    pub thumb_switches: ThumbSwitches,
    /// The connectors connecting the columns in the finger cluster.
    pub column_connectors: Vec<Connector>,
    /// The cluster connector connecting the finger and thumb cluster.
    pub cluster_connector: Connector,
    /// The FFC connector.
    pub ffc_connector: FfcConnector,
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

        let ffc_connector = FfcConnector::from_anchor(columns[cluster_connector_index].first());
        let cluster_connector = Connector::from_cluster_connector(
            &model.cluster_connector,
            ffc_connector.cluster_connector_start(),
        );

        let thumb_switches = ThumbSwitches::from_key_connectors(
            &model.thumb_key_connectors,
            cluster_connector.end_switch_position(),
        );

        Self {
            columns,
            thumb_switches,
            column_connectors,
            cluster_connector,
            ffc_connector,
        }
    }
}
