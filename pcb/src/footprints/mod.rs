mod ffc_connector;
mod switch;

use serde::Serialize;

use crate::{
    kicad_pcb::Net,
    primitives::{Point, Position, Size, Uuid},
    size,
    unit::{IntoUnit, Length},
};

pub use ffc_connector::FfcConnector;
pub use switch::Switch;

#[derive(Serialize)]
pub struct Footprint(&'static str, FootprintSettings);

#[derive(Serialize)]
struct FootprintSettings {
    layer: &'static str,
    uuid: Uuid,
    at: Position,
    descr: &'static str,
    tags: &'static str,
    #[serde(rename = "property_")]
    properties: Vec<Property>,
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
struct Property(&'static str, String, PropertySettings);

impl Property {
    /// Creates a new property at the given position.
    fn new(key: &'static str, value: String, position: Position) -> Self {
        Self(
            key,
            value,
            PropertySettings {
                at: position,
                layer: "F.Fab",
                uuid: Uuid::new(),
                effects: Effects::default(),
            },
        )
    }
}

#[derive(Serialize)]
struct PropertySettings {
    at: Position,
    layer: &'static str,
    uuid: Uuid,
    effects: Effects,
}

#[derive(Serialize, Default)]
struct Effects {
    font: Font,
}

#[derive(Serialize)]
struct Font {
    size: Size,
    thickness: Length,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            size: size!(1.0, 1.0),
            thickness: 0.15.mm(),
        }
    }
}

#[derive(Serialize)]
struct Line {
    start: Point,
    end: Point,
    stroke: Stroke,
    layer: &'static str,
    uuid: Uuid,
}

impl Line {
    /// Creates a new line with the given start and end points on the given layer.
    fn new(start: Point, end: Point, layer: &'static str) -> Self {
        let width = match layer {
            "F.SilkS" | "B.SilkS" => 0.12,
            "F.CrtYd" | "B.CrtYd" => 0.05,
            _ => 0.1,
        }
        .mm();

        Self {
            start,
            end,
            stroke: Stroke {
                width,
                stroke_type: StrokeType::Solid,
            },
            layer,
            uuid: Uuid::new(),
        }
    }
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
struct Pad(
    &'static str,
    PadType,
    PadShape,
    PadSettings,
    Option<RoundRectSettings>,
    Option<NetSettings>,
    PadUuid,
);

impl Pad {
    /// Creates a new pad.
    fn new(
        name: &'static str,
        pad_type: PadType,
        shape: PadShape,
        position: Position,
        size: Size,
        drill: Length,
        net: Option<Net>,
    ) -> Self {
        let roundrect_settings = if let PadShape::Roundrect = shape {
            Some(RoundRectSettings {
                roundrect_rratio: 0.25.mm(),
            })
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
            },
            roundrect_settings,
            net.map(|net| NetSettings { net }),
            PadUuid { uuid: Uuid::new() },
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
    Oval,
    Roundrect,
}

#[derive(Serialize)]
struct PadSettings {
    at: Position,
    size: Size,
    drill: Length,
    layers: (&'static str, &'static str),
    remove_unused_layers: bool,
}

#[derive(Serialize)]
struct RoundRectSettings {
    roundrect_rratio: Length,
}

#[derive(Serialize)]
struct NetSettings {
    net: Net,
}

#[derive(Serialize)]
struct PadUuid {
    uuid: Uuid,
}

/// Creates a vector of lines with the given start and end points at the front and back of the PCB.
#[macro_export]
macro_rules! lines_front_back {
    ($(($start:expr, $end:expr, $layer_type:literal)),* $(,)?) => {
        vec![
            $(
                $crate::footprints::Line::new($start, $end, concat!("F.", $layer_type)),
                $crate::footprints::Line::new($start, $end, concat!("B.", $layer_type)),
            )*
        ]
    };
}
