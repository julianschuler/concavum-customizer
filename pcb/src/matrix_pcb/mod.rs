mod builder;
mod connector;
mod features;
mod nets;

use std::iter::once;

use config::Config;

use crate::{
    kicad_pcb::KicadPcb,
    primitives::{Point, Position},
    unit::Length,
};

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

/// Adds the path given by the points to the outline.
fn add_outline_path(pcb: &mut KicadPcb, points: &[Point]) {
    for window in points.windows(2) {
        if window[0] != window[1] {
            pcb.add_graphical_line(window[0], window[1], OUTLINE_WIDTH, OUTLINE_LAYER);
        }
    }
}

/// Adds the polygon given by the points to the outline.
///
/// # Panic
///
/// Panics if there are less than three points given.
fn add_outline_polygon(pcb: &mut KicadPcb, points: &[Point]) {
    assert!(points.len() >= 3);

    let first = *points.first().expect("there are at least three vertices");
    let last = *points.last().expect("there are at least three vertices");

    for window in points.windows(2).chain(once([last, first].as_slice())) {
        if window[0] != window[1] {
            pcb.add_graphical_line(window[0], window[1], OUTLINE_WIDTH, OUTLINE_LAYER);
        }
    }
}
