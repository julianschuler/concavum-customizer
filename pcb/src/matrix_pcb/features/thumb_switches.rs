use model::matrix_pcb::{Segment, ThumbKeyConnectors, CONNECTOR_WIDTH, PAD_SIZE};

use crate::{
    kicad_pcb::KicadPcb,
    matrix_pcb::{add_outline_path, add_outline_polygon},
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
        let first = self
            .0
            .first()
            .copied()
            .expect("there is always at least one thumb key");
        let last = self
            .0
            .last()
            .copied()
            .expect("there is always at least one thumb key");

        let outline_points = [
            first + point!(PAD_SIZE.x / 2.0 - CONNECTOR_WIDTH, -PAD_SIZE.y / 2.0),
            first + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
            first + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
            last + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
            last + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
            first + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
        ];
        add_outline_path(pcb, &outline_points);

        for window in self.0.windows(2) {
            let position = window[0];
            let next_position = window[1];

            let cutout_points = [
                position + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0 - CONNECTOR_WIDTH),
                position + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0 + CONNECTOR_WIDTH),
                next_position + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0 + CONNECTOR_WIDTH),
                next_position + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0 - CONNECTOR_WIDTH),
            ];
            add_outline_polygon(pcb, &cutout_points);
        }
    }
}
