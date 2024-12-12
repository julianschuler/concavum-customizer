mod builder;
mod connector;
mod features;
mod nets;

use config::Config;

use crate::{kicad_pcb::KicadPcb, primitives::Position, unit::Length};

use builder::Builder;

/// The name of the outline layer.
const OUTLINE_LAYER: &str = "Edge.Cuts";
/// The width of the outline.
const OUTLINE_WIDTH: Length = Length::mm(0.05);
/// The position of the first home row key.
const ORIGIN_POSITION: Position = Position::new(Length::mm(100.0), Length::mm(100.0), None);

/// A PCB connecting the keys to each other in a matrix.
pub struct MatrixPcb(KicadPcb);

impl MatrixPcb {
    /// Creates a new matrix PCB from the given configuration.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        Self(Builder::from_config(config).build())
    }

    /// Serializes the matrix PCB to the KiCAD board file format.
    #[must_use]
    pub fn to_kicad_board(&self) -> String {
        self.0.to_board_file()
    }
}
