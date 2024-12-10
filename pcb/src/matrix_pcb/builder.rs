use config::Config;
use model::{
    matrix_pcb::{MatrixPcb as Model, THICKNESS},
    KeyPositions,
};

use crate::{
    footprints::{FpcConnector, Switch},
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{nets::Nets, switch_positions::SwitchPositions},
    position,
    primitives::Position,
    unit::IntoUnit,
};

/// A builder for the matrix PCB.
pub struct Builder {
    pcb: KicadPcb,
    model: Model,
    cluster_connector_index: usize,
    home_row_index: usize,
    switch_count: usize,
}

impl Builder {
    /// Creates a new builder from the given config.
    pub fn from_config(config: &Config) -> Self {
        let pcb = KicadPcb::new(THICKNESS.mm());

        let key_positions = KeyPositions::from_config(config);
        let model = Model::from_positions(&key_positions);

        #[allow(clippy::cast_sign_loss)]
        let home_row_index = key_positions.columns.home_row_index as usize;
        let cluster_connector_index =
            usize::from(config.finger_cluster.columns.left_side_column.active);

        Self {
            pcb,
            model,
            cluster_connector_index,
            home_row_index,
            switch_count: 0,
        }
    }

    /// Builds the matrix PCB.
    pub fn build(mut self) -> KicadPcb {
        let switch_positions = SwitchPositions::from_model(
            &self.model,
            self.home_row_index,
            self.cluster_connector_index,
        );
        let nets = Nets::create(&mut self.pcb);

        self.add_switches(&switch_positions, &nets);
        self.add_fpc_connector(&switch_positions, &nets);

        self.pcb
    }

    /// Adds the switches for the finger and thumb cluster to the PCB.
    fn add_switches(&mut self, switch_positions: &SwitchPositions, nets: &Nets) {
        for (&position, column_net) in switch_positions
            .thumb_switches
            .positions()
            .iter()
            .zip(&nets.columns)
        {
            self.add_switch(position, nets.rows[0].clone(), column_net.clone());
        }
        for (column, column_net) in switch_positions.columns.iter().zip(&nets.columns) {
            for (&position, row_net) in column.positions().zip(nets.rows.iter().skip(1)) {
                self.add_switch(position, row_net.clone(), column_net.clone());
            }
        }
    }

    /// Adds a single switch at the given position and with the given row and column net to the PCB.
    fn add_switch(&mut self, position: Position, row_net: Net, column_net: Net) {
        self.switch_count += 1;
        let reference = format!("SW{}", self.switch_count);
        let internal_net = self.pcb.create_net(reference.clone());

        let switch = Switch::new(reference, position, row_net, column_net, internal_net);
        self.pcb.add_footprint(switch.into());
    }

    /// Adds the FPC connector to the PCB.
    fn add_fpc_connector(&mut self, switch_positions: &SwitchPositions, nets: &Nets) {
        let fpc_connector_nets = [
            nets.rows[0].clone(),
            nets.columns[0].clone(),
            nets.columns[1].clone(),
            nets.columns[2].clone(),
            nets.columns[3].clone(),
            nets.columns[4].clone(),
            nets.rows[1].clone(),
            nets.rows[2].clone(),
            nets.rows[3].clone(),
            nets.rows[4].clone(),
            nets.rows[5].clone(),
            nets.columns[5].clone(),
        ];

        let fpc_connector_position = switch_positions.columns[self.cluster_connector_index].first()
            + position!(0, 5.5, None);
        let fpc_connector =
            FpcConnector::new("J1".to_owned(), fpc_connector_position, fpc_connector_nets);
        self.pcb.add_footprint(fpc_connector.into());
    }
}
