use model::matrix_pcb::{Segment, SideColumnConnector, PAD_SIZE};

use crate::{
    position,
    primitives::Position,
    unit::{IntoUnit, Length},
};

/// A straight connector described by a position and a length.
pub struct StraightConnector {
    start: Position,
    length: Length,
}

impl StraightConnector {
    /// Creates a straight connector from a side column connector and a switch position.
    pub fn from_side_column_connector(
        side_column_connector: &SideColumnConnector,
        switch_position: Position,
    ) -> Self {
        let start = switch_position + position!(PAD_SIZE.x / 2.0, 0, None);
        let length = side_column_connector.length().mm();

        Self { start, length }
    }

    /// Returns the end position of the connector.
    pub fn end_position(&self) -> Position {
        self.start + Position::new(self.length, 0.mm(), None)
    }

    /// Returns the position of the switch at the end of the connector
    pub fn end_switch_position(&self) -> Position {
        self.end_position() + position!(PAD_SIZE.x / 2.0, 0, None)
    }
}
