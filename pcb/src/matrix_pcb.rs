use config::Config;

use crate::{footprints::Key, kicad_pcb::KicadPcb, primitives::Position, unit::IntoUnit};

/// A key matrix PCB generated from a configuration.
pub struct MatrixPcb {
    pcb: KicadPcb,
}

impl MatrixPcb {
    /// Creates a new matrix PCB from the given configuration.
    #[must_use]
    pub fn from_config(_config: &Config) -> Self {
        let mut pcb = KicadPcb::new(0.6.mm());
        let key = Key::new(Position::new(10.0.mm(), 5.0.mm(), Some(0.0.deg())));

        pcb.add_footprint(key.into());

        Self { pcb }
    }

    /// Serializes the matrix PCB to the KiCAD board file format.
    #[must_use]
    pub fn to_kicad_board(&self) -> String {
        self.pcb.to_board_file()
    }
}
