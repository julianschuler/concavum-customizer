use std::iter::once;

use config::Config;
use model::{
    matrix_pcb::{MatrixPcb as Model, THICKNESS},
    KeyPositions,
};

use crate::{
    footprints::{FfcConnector, Switch},
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{
        connector::Connector,
        features::{Column, Features, ThumbSwitches},
        nets::Nets,
        AddPath, BOTTOM_LAYER, TOP_LAYER, TRACK_CLEARANCE, TRACK_WIDTH,
    },
    point,
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
        let features = Features::from_model(
            &self.model,
            self.home_row_index,
            self.cluster_connector_index,
        );
        let nets = Nets::create(&mut self.pcb);

        self.add_outline(&features);
        self.add_switches(&features.columns, &features.thumb_switches, &nets);
        self.add_ffc_connector(features.ffc_connector.position(), &nets);

        self.add_cluster_connector_tracks(&features, &nets);

        self.pcb
    }

    /// Adds the outline to the PCB using the given features.
    fn add_outline(&mut self, features: &Features) {
        let column_connectors: Vec<_> = once(None)
            .chain(features.column_connectors.iter().map(Option::Some))
            .chain(once(None))
            .collect();

        for (i, (window, column)) in column_connectors
            .windows(2)
            .zip(&features.columns)
            .enumerate()
        {
            column.add_outline(
                &mut self.pcb,
                window[0].map(Connector::end_position),
                window[1].map(Connector::start_position),
                i == self.cluster_connector_index,
            );
        }
        features.thumb_switches.add_outline(&mut self.pcb);

        for column_connector in &features.column_connectors {
            column_connector.add_outline(&mut self.pcb);
        }
        features.cluster_connector.add_outline(&mut self.pcb);
        features.ffc_connector.add_outline(&mut self.pcb);
    }

    /// Adds the switches for the finger and thumb cluster to the PCB.
    fn add_switches(&mut self, columns: &[Column], thumb_switches: &ThumbSwitches, nets: &Nets) {
        for (&position, column_net) in thumb_switches.positions().iter().zip(&nets.columns) {
            self.add_switch(position, nets.rows[0].clone(), column_net.clone());
        }
        for (column, column_net) in columns.iter().zip(&nets.columns) {
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

        let column_track_points = [
            position + point!(-2.54, -5.08),
            position + point!(-3.81, -2.54),
        ];
        let internal_track_points = [
            position + point!(2.54, -5.08),
            position + point!(3.81, -2.54),
            position + point!(3.81, 0.24),
            position + point!(1.65, 2.4),
            position + point!(1.65, 3.41),
        ];
        self.pcb
            .add_track(&column_track_points, TOP_LAYER, &column_net);
        self.pcb
            .add_track(&internal_track_points, TOP_LAYER, &internal_net);

        let switch = Switch::new(reference, position, row_net, column_net, internal_net);
        self.pcb.add_footprint(switch.into());
    }

    /// Adds the FFC connector to the PCB.
    fn add_ffc_connector(&mut self, position: Position, nets: &Nets) {
        let ffc_connector_nets = [
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

        let ffc_connector = FfcConnector::new("J1".to_owned(), position, ffc_connector_nets);
        self.pcb.add_footprint(ffc_connector.into());
    }

    /// Adds the cluster connector tracks to the PCB.
    fn add_cluster_connector_tracks(&mut self, features: &Features, nets: &Nets) {
        features.cluster_connector.add_track(
            &mut self.pcb,
            0.mm(),
            TRACK_WIDTH,
            TOP_LAYER,
            &nets.rows[0],
        );

        let track_count = features.thumb_switches.positions().len();
        let track_spacing = TRACK_WIDTH + TRACK_CLEARANCE;

        for i in 0..track_count {
            #[allow(clippy::cast_precision_loss)]
            let offset = ((track_count - 1) as f32 / 2.0 - i as f32) * track_spacing;

            features.cluster_connector.add_track(
                &mut self.pcb,
                offset,
                TRACK_WIDTH,
                BOTTOM_LAYER,
                &nets.columns[i],
            );
        }
    }
}
