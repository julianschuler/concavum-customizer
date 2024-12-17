use model::matrix_pcb::{CONNECTOR_WIDTH, FFC_PAD_OFFSET, FFC_PAD_SIZE, PAD_SIZE};

use crate::{
    kicad_pcb::KicadPcb, matrix_pcb::AddPath, point, position, primitives::Position, unit::IntoUnit,
};

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

    /// Adds the outline of the FFC connector to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        for sign in [-1.0, 1.0] {
            let ffc_pad_top_offset = FFC_PAD_OFFSET - FFC_PAD_SIZE.y / 2.0;
            let ffc_pad_bottom_offset = FFC_PAD_OFFSET + FFC_PAD_SIZE.y / 2.0;

            let outline_points = [
                self.anchor + point!(sign * CONNECTOR_WIDTH / 2.0, ffc_pad_bottom_offset),
                self.anchor + point!(sign * FFC_PAD_SIZE.x / 2.0, ffc_pad_bottom_offset),
                self.anchor + point!(sign * FFC_PAD_SIZE.x / 2.0, ffc_pad_top_offset),
                self.anchor + point!(sign * PAD_SIZE.x / 2.0, ffc_pad_top_offset),
                self.anchor + point!(sign * PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
            ];

            pcb.add_outline_path(&outline_points);
        }
    }
}
