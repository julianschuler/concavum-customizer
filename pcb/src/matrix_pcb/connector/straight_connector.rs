use model::matrix_pcb::{Segment, SideColumnConnector, CONNECTOR_WIDTH, PAD_SIZE};

use crate::{
    kicad_pcb::KicadPcb,
    matrix_pcb::{OUTLINE_LAYER, OUTLINE_WIDTH},
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

    /// Adds the outline of the connector to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        let offset = position!(0, CONNECTOR_WIDTH / 2.0, None);

        let start_top = self.start + offset;
        let start_bottom = self.start - offset;
        let end_top = self.end_position() + offset;
        let end_bottom = self.end_position() - offset;

        pcb.add_graphical_line(
            start_top.point(),
            end_top.point(),
            OUTLINE_WIDTH,
            OUTLINE_LAYER,
        );
        pcb.add_graphical_line(
            start_bottom.point(),
            end_bottom.point(),
            OUTLINE_WIDTH,
            OUTLINE_LAYER,
        );
    }
}
