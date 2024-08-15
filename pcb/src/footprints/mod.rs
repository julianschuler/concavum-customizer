mod key;

use serde::Serialize;

use crate::{
    primitives::{Point, Position, Size, Uuid},
    unit::{IntoUnit, Length},
};

pub use key::Key;

#[derive(Serialize)]
pub struct Footprint(&'static str, FootprintSettings);

#[derive(Serialize)]
struct FootprintSettings {
    layer: &'static str,
    uuid: Uuid,
    at: Position,
    descr: &'static str,
    tags: &'static str,
    attr: Attribute,
    fp_lines: Vec<Line>,
    pads: Vec<Pad>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Attribute {
    ThroughHole,
}

#[derive(Serialize)]
struct Line {
    start: Point,
    end: Point,
    layer: &'static str,
    stroke: Stroke,
}

#[derive(Serialize)]
struct Stroke {
    width: Length,
    #[serde(rename = "type")]
    stroke_type: StrokeType,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum StrokeType {
    Solid,
}

#[derive(Serialize)]
struct Pad(&'static str, PadType, PadShape, PadSettings);

impl Pad {
    /// Creates a new pad.
    fn new(
        name: &'static str,
        pad_type: PadType,
        shape: PadShape,
        position: Position,
        size: Size,
        drill: Length,
    ) -> Self {
        let roundrect_rratio = if let PadShape::Roundrect = shape {
            Some(0.25.mm())
        } else {
            None
        };

        Self(
            name,
            pad_type,
            shape,
            PadSettings {
                at: position,
                size,
                drill,
                layers: ("*.Cu", "*.Mask"),
                remove_unused_layers: false,
                roundrect_rratio,
                uuid: Uuid::new(),
            },
        )
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum PadType {
    NpThruHole,
    ThruHole,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum PadShape {
    Circle,
    Roundrect,
}

#[derive(Serialize)]
struct PadSettings {
    at: Position,
    size: Size,
    drill: Length,
    layers: (&'static str, &'static str),
    remove_unused_layers: bool,
    roundrect_rratio: Option<Length>,
    uuid: Uuid,
}
