use model::matrix_pcb::{Segment, ThumbKeyConnectors, CONNECTOR_WIDTH, PAD_SIZE};

use crate::{
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{nets::Nets, AddPath, TOP_LAYER},
    point, position,
    primitives::Position,
};

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

    /// Adds the outline for the thumb switches to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        let first = {
            let this = &self;
            this.0
                .first()
                .copied()
                .expect("there is always at least one thumb key")
        };
        let last = {
            let this = &self;
            this.0
                .last()
                .copied()
                .expect("there is always at least one thumb key")
        };

        let outline_points = [
            first + point!(PAD_SIZE.x / 2.0 - CONNECTOR_WIDTH, -PAD_SIZE.y / 2.0),
            first + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
            first + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
            last + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
            last + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
            first + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
        ];
        pcb.add_outline_path(&outline_points);

        for window in self.0.windows(2) {
            let position = window[0];
            let next_position = window[1];

            let cutout_points = [
                position + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0 - CONNECTOR_WIDTH),
                position + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0 + CONNECTOR_WIDTH),
                next_position + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0 + CONNECTOR_WIDTH),
                next_position + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0 - CONNECTOR_WIDTH),
            ];
            pcb.add_outline_polygon(&cutout_points);
        }
    }

    /// Adds the tracks for the thumb switches to the PCB.
    pub fn add_tracks(&self, pcb: &mut KicadPcb, nets: &Nets) {
        self.add_row_track(pcb, &nets.rows[0]);
    }

    /// Adds the track connecting the row of the thumb switches.
    fn add_row_track(&self, pcb: &mut KicadPcb, row_net: &Net) {
        let (first, rest) = self
            .0
            .split_first()
            .expect("there is always at least one thumb switch");

        if let Some((last, rest)) = rest.split_last() {
            let y_offset = (PAD_SIZE.y - CONNECTOR_WIDTH) / 2.0;
            let track_points = [
                *first + point!(-1.65, 3.4),
                *first + point!(-1.65, 4.4),
                *first + point!(-1.65 + (y_offset - 4.4), y_offset),
                *last + point!(-1.65 - (y_offset - 4.4), y_offset),
                *last + point!(-1.65, 4.4),
                *last + point!(-1.65, 3.4),
            ];
            pcb.add_track(&track_points, TOP_LAYER, row_net);

            let track_points = [
                *first + point!((PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0, y_offset),
                *first + point!((PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0, -PAD_SIZE.y / 2.0),
            ];
            pcb.add_track(&track_points, TOP_LAYER, row_net);

            for position in rest {
                let track_points = [
                    *position + point!(-1.65, 3.4),
                    *position + point!(-1.65, y_offset),
                ];
                pcb.add_track(&track_points, TOP_LAYER, row_net);
            }
        } else {
            let chamfer_depth = 3.0;
            let track_points = [
                *first + point!(-1.65, 3.4),
                *first + point!(-1.65, 4.4),
                *first + point!((PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0 - chamfer_depth, 4.4),
                *first + point!((PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0, 4.4 - chamfer_depth),
                *first + point!((PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0, -PAD_SIZE.y / 2.0),
            ];

            pcb.add_track(&track_points, TOP_LAYER, row_net);
        };
    }
}
