use model::matrix_pcb::{FFC_PAD_OFFSET, FFC_PAD_SIZE};

use crate::{position, primitives::Position, unit::IntoUnit};

/// An FFC connector.
pub struct FfcConnector {
    anchor: Position,
}

impl FfcConnector {
    /// Creates a new FFC connector from the given anchor switch position.
    pub fn from_anchor(anchor: Position) -> Self {
        Self { anchor }
    }

    /// Calculates the start of the cluster connector.
    pub fn cluster_connector_start(&self) -> Position {
        self.anchor + position!(0, FFC_PAD_OFFSET + FFC_PAD_SIZE.y / 2.0, Some(-90.deg()))
    }

    /// Returns the position of the FFC connector.
    pub fn position(&self) -> Position {
        const FFC_CONNECTOR_OFFSET: f64 = 5.5;

        self.anchor + position!(0, FFC_CONNECTOR_OFFSET, None)
    }
}
