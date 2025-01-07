use model::matrix_pcb::{Segment, SideColumnConnector, CONNECTOR_WIDTH, PAD_SIZE};

use crate::{
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{OUTLINE_LAYER, OUTLINE_WIDTH, TRACK_WIDTH},
    point, position,
    primitives::Position,
    unit::Length,
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
        let length = side_column_connector.length().into();

        Self { start, length }
    }

    /// Returns the start position of the connector
    pub fn start_position(&self) -> Position {
        self.start
    }

    pub fn start_switch_position(&self) -> Position {
        self.start_position() + position!(-PAD_SIZE.x / 2.0, 0, None)
    }

    /// Returns the end position of the connector.
    pub fn end_position(&self) -> Position {
        self.start + position!(self.length, 0, None)
    }

    /// Returns the position of the switch at the end of the connector
    pub fn end_switch_position(&self) -> Position {
        self.end_position() + position!(PAD_SIZE.x / 2.0, 0, None)
    }

    /// Adds the outline of the connector to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        let offset = point!(0, CONNECTOR_WIDTH / 2.0);

        let start_top = self.start + offset;
        let start_bottom = self.start - offset;
        let end_top = self.end_position() + offset;
        let end_bottom = self.end_position() - offset;

        pcb.add_graphical_line(start_top, end_top, OUTLINE_WIDTH, OUTLINE_LAYER);
        pcb.add_graphical_line(start_bottom, end_bottom, OUTLINE_WIDTH, OUTLINE_LAYER);
    }

    /// Adds a track to the PCB with the given offset to the center.
    pub fn add_track(&self, pcb: &mut KicadPcb, offset: Length, layer: &'static str, net: &Net) {
        let offset = point!(0, offset);
        let start = self.start + offset;
        let end = self.end_position() + offset;

        pcb.add_segment(start, end, TRACK_WIDTH, layer, net);
    }
}
