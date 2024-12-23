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
        centered_track_offset,
        connector::Connector,
        features::{Column, Features, ThumbSwitches},
        nets::Nets,
        AddPath, BOTTOM_LAYER, TOP_LAYER,
    },
    point,
    primitives::Position,
};

/// A builder for the matrix PCB.
pub struct Builder {
    pcb: KicadPcb,
    model: Model,
    cluster_connector_index: usize,
    home_row_index: usize,
    row_count: usize,
    column_count: usize,
    thumb_switch_count: usize,
    switch_count: usize,
}

impl Builder {
    /// Creates a new builder from the given config.
    pub fn from_config(config: &Config) -> Self {
        let pcb = KicadPcb::new(THICKNESS.into());

        let key_positions = KeyPositions::from_config(config);
        let model = Model::from_positions(&key_positions);

        #[allow(clippy::cast_sign_loss)]
        let home_row_index = key_positions.columns.home_row_index as usize;
        let cluster_connector_index =
            usize::from(config.finger_cluster.columns.left_side_column.active);
        #[allow(clippy::cast_sign_loss)]
        let row_count = i8::from(config.finger_cluster.rows) as usize;
        let column_count = model.column_key_connectors.len();
        #[allow(clippy::cast_sign_loss)]
        let thumb_switch_count = i8::from(config.thumb_cluster.keys) as usize;

        Self {
            pcb,
            model,
            cluster_connector_index,
            home_row_index,
            row_count,
            column_count,
            thumb_switch_count,
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

        self.add_column_connector_tracks(&features, &nets);
        self.add_cluster_connector_tracks(&features, &nets);

        self.add_ffc_connector_tracks(&features, &nets);
        features.thumb_switches.add_tracks(&mut self.pcb, &nets);

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
            position + point!(2.54, -5.08),
            position + point!(3.81, -2.54),
        ];
        let internal_track_points = [
            position + point!(-2.54, -5.08),
            position + point!(-3.81, -2.54),
            position + point!(-3.81, 0.24),
            position + point!(-1.65, 2.4),
            position + point!(-1.65, 3.4),
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
            nets.columns[0].clone(),
            nets.rows[5].clone(),
            nets.rows[4].clone(),
            nets.rows[3].clone(),
            nets.rows[2].clone(),
            nets.rows[1].clone(),
            nets.columns[1].clone(),
            nets.columns[2].clone(),
            nets.columns[3].clone(),
            nets.columns[4].clone(),
            nets.columns[5].clone(),
            nets.rows[0].clone(),
        ];

        let ffc_connector = FfcConnector::new("J1".to_owned(), position, ffc_connector_nets);
        self.pcb.add_footprint(ffc_connector.into());
    }

    /// Adds the column connector tracks to the PCB.
    fn add_column_connector_tracks(&mut self, features: &Features, nets: &Nets) {
        let bottom_nets = &nets.rows[1..=self.row_count];

        for (i, connector) in features.column_connectors.iter().rev().enumerate() {
            let top_nets = if i >= self.column_count - self.cluster_connector_index - 1 {
                &nets.columns[..self.cluster_connector_index]
            } else {
                &nets.columns[self.column_count - i - 1..self.column_count]
            };

            self.add_connector_tracks(connector, TOP_LAYER, top_nets);
            self.add_connector_tracks(connector, BOTTOM_LAYER, bottom_nets);
        }
    }

    /// Adds the cluster connector tracks to the PCB.
    fn add_cluster_connector_tracks(&mut self, features: &Features, nets: &Nets) {
        features
            .cluster_connector
            .add_track(&mut self.pcb, 0.into(), TOP_LAYER, &nets.rows[0]);

        let nets = &nets.columns[..self.thumb_switch_count];
        self.add_connector_tracks(&features.cluster_connector, BOTTOM_LAYER, nets);
    }

    /// Adds a connector track for each of the given nets on the given layer.
    fn add_connector_tracks(&mut self, connector: &Connector, layer: &'static str, nets: &[Net]) {
        let sign = if layer == TOP_LAYER { 1 } else { -1 };

        for (i, net) in nets.iter().enumerate() {
            let offset = sign * centered_track_offset(i, nets.len());

            connector.add_track(&mut self.pcb, offset, layer, net);
        }
    }

    /// Adds the tracks for the FFC connector to the PCB.
    fn add_ffc_connector_tracks(&mut self, features: &Features, nets: &Nets) {
        let left_column_connector = if self.cluster_connector_index > 0 {
            features
                .column_connectors
                .get(self.cluster_connector_index - 1)
        } else {
            None
        };
        let right_column_connector = features.column_connectors.get(self.cluster_connector_index);

        features.ffc_connector.add_row_tracks(
            &mut self.pcb,
            &nets.columns[1..],
            &features.columns[self.cluster_connector_index],
        );
        features.ffc_connector.add_column_tracks(
            &mut self.pcb,
            nets,
            self.row_count,
            self.column_count,
            left_column_connector.map(Connector::end_position),
            right_column_connector.map(Connector::start_position),
        );
        features.ffc_connector.add_cluster_connector_tracks(
            &mut self.pcb,
            nets,
            self.thumb_switch_count,
        );
    }
}
