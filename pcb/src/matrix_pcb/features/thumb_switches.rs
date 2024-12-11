use model::matrix_pcb::{Segment, ThumbKeyConnectors, PAD_SIZE};

use crate::{position, primitives::Position};

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
