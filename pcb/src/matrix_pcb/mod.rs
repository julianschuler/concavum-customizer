mod builder;
mod connector;
mod features;
mod nets;

use std::iter::once;

use config::Config;

use crate::{
    kicad_pcb::{KicadPcb, Net},
    primitives::{Point, Position},
    unit::Length,
};

use builder::Builder;

/// The name of the top copper layer.
const TOP_LAYER: &str = "F.Cu";
/// The name of the bottom copper layer.
const BOTTOM_LAYER: &str = "B.Cu";
/// The name of the outline layer.
const OUTLINE_LAYER: &str = "Edge.Cuts";
/// The width of the outline.
const OUTLINE_WIDTH: Length = Length::mm(0.05);
/// The position of the first home row key.
const ORIGIN_POSITION: Position = Position::new(Length::mm(100.0), Length::mm(100.0), None);
/// The track width.
const TRACK_WIDTH: Length = Length::mm(0.15);
/// The track clearance.
const TRACK_CLEARANCE: Length = Length::mm(0.15);

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

/// A trait for adding paths and polygons.
trait AddPath {
    /// Adds the path given by the points to the outline of `self`.
    fn add_outline_path(&mut self, points: &[Point]);

    /// Adds the polygon given by the points to the outline of `self`.
    fn add_outline_polygon(&mut self, points: &[Point]);

    /// Adds a track along the path given by the points to `self`.
    fn add_track(&mut self, points: &[Point], layer: &'static str, net: &Net);
}

impl AddPath for KicadPcb {
    fn add_outline_path(&mut self, points: &[Point]) {
        for window in points.windows(2) {
            if window[0] != window[1] {
                self.add_graphical_line(window[0], window[1], OUTLINE_WIDTH, OUTLINE_LAYER);
            }
        }
    }

    fn add_outline_polygon(&mut self, points: &[Point]) {
        assert!(points.len() >= 3);

        let first = *points.first().expect("there are at least three vertices");
        let last = *points.last().expect("there are at least three vertices");

        for window in points.windows(2).chain(once([last, first].as_slice())) {
            if window[0] != window[1] {
                self.add_graphical_line(window[0], window[1], OUTLINE_WIDTH, OUTLINE_LAYER);
            }
        }
    }

    fn add_track(&mut self, points: &[Point], layer: &'static str, net: &Net) {
        for window in points.windows(2) {
            if window[0] != window[1] {
                self.add_segment(window[0], window[1], TRACK_WIDTH, layer, net);
            }
        }
    }
}
