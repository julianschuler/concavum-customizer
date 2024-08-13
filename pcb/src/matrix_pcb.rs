use config::Config;

use crate::kicad_pcb::KicadPcb;

/// A key matrix PCB generated from a configuration.
pub struct MatrixPcb {
    inner: KicadPcb,
}

impl MatrixPcb {
    /// Creates a new matrix PCB from the given configuration.
    #[must_use]
    pub fn from_config(_config: &Config) -> Self {
        let inner = KicadPcb::new(0.6);

        Self { inner }
    }

    /// Serializes the matrix PCB to the KiCAD board file format.
    #[must_use]
    pub fn to_kicad_board(&self) -> String {
        self.inner.to_board_file()
    }
}
