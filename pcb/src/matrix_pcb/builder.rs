use std::iter::once;

use config::Config;
use model::{
    matrix_pcb::{MatrixPcb as Model, PAD_SIZE, ROUTER_BIT_DIAMETER, THICKNESS},
    KeyPositions,
};

use crate::{
    footprints::{FfcConnector, Switch, Tab},
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{
        centered_track_offset,
        connector::Connector,
        features::{Column, Features, ThumbSwitches},
        nets::Nets,
        AddPath, BOTTOM_LAYER, TOP_LAYER,
    },
    point, position,
    primitives::Position,
    unit::{IntoAngle, Length},
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
        let nets = Nets::create(
            &mut self.pcb,
            self.row_count,
            self.column_count,
            self.thumb_switch_count,
            self.home_row_index,
        );

        self.add_outline(&features);
        self.add_switches(&features.columns, &features.thumb_switches, &nets);
        self.add_ffc_connector(features.ffc_connector.position(), &nets);

        self.add_column_connector_tracks(&features, &nets);
        self.add_cluster_connector_tracks(&features, &nets);

        self.add_ffc_connector_tracks(&features, &nets);
        self.add_column_tracks(&features, &nets);
        self.add_row_tracks(&features, &nets);
        features.thumb_switches.add_tracks(&mut self.pcb, &nets);

        self.add_tabs(&features);

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
        for (&position, column_net) in thumb_switches.positions().iter().zip(nets.thumb_columns()) {
            self.add_switch(position, nets.thumb_row().clone(), column_net.clone());
        }
        for (column, column_net) in columns.iter().zip(nets.columns()) {
            for (&position, row_net) in column.positions().zip(nets.finger_rows()) {
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
        let ffc_connector = FfcConnector::new("J1".to_owned(), position, nets.ffc_connector_nets());

        self.pcb.add_footprint(ffc_connector.into());
    }

    /// Adds the column connector tracks to the PCB.
    fn add_column_connector_tracks(&mut self, features: &Features, nets: &Nets) {
        let bottom_nets = &nets.finger_rows();

        for (i, connector) in features.column_connectors.iter().rev().enumerate() {
            let top_nets = if i >= self.column_count - self.cluster_connector_index - 1 {
                &nets.columns()[..self.cluster_connector_index]
            } else {
                &nets.columns()[self.column_count - i - 1..]
            };

            self.add_connector_tracks(connector, TOP_LAYER, top_nets);
            self.add_connector_tracks(connector, BOTTOM_LAYER, bottom_nets);
        }
    }

    /// Adds the cluster connector tracks to the PCB.
    fn add_cluster_connector_tracks(&mut self, features: &Features, nets: &Nets) {
        features.cluster_connector.add_track(
            &mut self.pcb,
            centered_track_offset(0, 2),
            TOP_LAYER,
            nets.thumb_row(),
        );
        features.cluster_connector.add_track(
            &mut self.pcb,
            centered_track_offset(1, 2),
            TOP_LAYER,
            &nets.columns()[0],
        );

        let nets = &nets.thumb_columns()[1..];
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
            nets.finger_rows(),
            &features.columns[self.cluster_connector_index],
        );
        features.ffc_connector.add_column_tracks(
            &mut self.pcb,
            nets,
            self.row_count,
            left_column_connector.map(Connector::end_position),
            right_column_connector.map(Connector::start_position),
        );
        features.ffc_connector.add_cluster_connector_tracks(
            &mut self.pcb,
            nets,
            self.thumb_switch_count,
        );
    }

    /// Adds the tracks for the columns to the PCB.
    fn add_column_tracks(&mut self, features: &Features, nets: &Nets) {
        for (column, net) in features.columns.iter().zip(nets.columns()) {
            column.add_switch_tracks(&mut self.pcb, net);
        }

        for (i, (window, column)) in features
            .column_connectors
            .windows(2)
            .zip(features.columns.iter().skip(1))
            .skip(self.cluster_connector_index)
            .rev()
            .enumerate()
        {
            let nets = &nets.columns()[self.column_count - 1 - i..];

            column.add_column_tracks(
                &mut self.pcb,
                nets,
                window[0].end_position(),
                window[1].start_position(),
            );
        }

        let (left_column_connectors, right_column_connectors) = features
            .column_connectors
            .split_at(self.cluster_connector_index);

        for left_column_connector in left_column_connectors {
            left_column_connector.add_left_column_track(&mut self.pcb, &nets.columns()[0]);
        }
        for (i, right_column_connector) in right_column_connectors.iter().rev().enumerate() {
            right_column_connector.add_right_column_track(
                &mut self.pcb,
                i + 1,
                &nets.columns()[self.column_count - 1 - i],
            );
        }
    }

    /// Adds the tracks for the rows to the PCB.
    fn add_row_tracks(&mut self, features: &Features, nets: &Nets) {
        let home_row_offset = centered_track_offset(self.home_row_index, self.row_count);
        for column_connector in &features.column_connectors {
            column_connector.add_home_row_tracks(&mut self.pcb, nets.home_row(), home_row_offset);
        }

        if let Some(column_connector) = features.column_connectors.first() {
            features
                .columns
                .first()
                .expect("there is always at least one column")
                .add_outer_column_row_tracks(
                    &mut self.pcb,
                    nets,
                    column_connector.start_attachment_side(),
                    true,
                );
        }
        if let Some(column_connector) = features.column_connectors.last() {
            features
                .columns
                .last()
                .expect("there is always at least one column")
                .add_outer_column_row_tracks(
                    &mut self.pcb,
                    nets,
                    column_connector.end_attachment_side(),
                    false,
                );
        }
    }

    /// Adds the tab markers to the PCB.
    fn add_tabs(&mut self, features: &Features) {
        let tab_offset = Length::new(PAD_SIZE.y / 2.0) + Length::EPSILON;
        let last_thumb_switch = features.thumb_switches.last();

        let upper_thumb_corner = last_thumb_switch + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0);
        let lower_thumb_corner = last_thumb_switch + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0);

        let minimum_x_value = upper_thumb_corner.x().max(lower_thumb_corner.x())
            + ROUTER_BIT_DIAMETER.into()
            + Tab::WIDTH / 2;

        for column in &features.columns {
            let tab = Tab::new(column.last() + position!(0, -tab_offset, Some(-90.deg())));

            self.pcb.add_footprint(tab.into());
        }

        for position in once(features.thumb_switches.first())
            .chain((features.thumb_switches.positions().len() > 1).then_some(last_thumb_switch))
            .chain(features.columns.iter().filter_map(|column| {
                (column.first().x() >= minimum_x_value).then_some(column.first())
            }))
        {
            let lower_tab = Tab::new(position + position!(0, tab_offset, Some(90.deg())));

            self.pcb.add_footprint(lower_tab.into());
        }
    }
}
