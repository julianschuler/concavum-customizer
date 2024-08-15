use crate::{
    footprints::{
        Attribute, Footprint, FootprintSettings, Line, Pad, PadShape, PadType, Stroke, StrokeType,
    },
    point, position,
    primitives::{Position, Uuid},
    size,
    unit::IntoUnit,
};

/// A footprint for a Cherry MX key switch.
pub struct Key {
    position: Position,
}

impl Key {
    /// Creates a new key at the given position.
    pub fn new(position: Position) -> Self {
        Self { position }
    }
}

impl From<Key> for Footprint {
    #[allow(clippy::too_many_lines)]
    fn from(key: Key) -> Self {
        let angle = key.position.angle();

        Self(
            "key_switch_dual_side_diode",
            FootprintSettings {
                layer: "F.Cu",
                uuid: Uuid::new(),
                at: key.position,
                descr: "Cherry MX key switch dual side with diode",
                tags: "Cherry MX key switch dual side with diode",
                attr: Attribute::ThroughHole,
                fp_lines: vec![Line {
                    start: point!(-2.25, 4.41),
                    end: point!(1.65, 4.41),
                    layer: "B.SilkS",
                    stroke: Stroke {
                        width: 0.12.mm(),
                        stroke_type: StrokeType::Solid,
                    },
                }],
                pads: vec![
                    Pad::new(
                        "",
                        PadType::NpThruHole,
                        PadShape::Circle,
                        position!(0.0, 0.0, angle),
                        size!(4.0, 4.0),
                        4.0.mm(),
                    ),
                    Pad::new(
                        "1",
                        PadType::ThruHole,
                        PadShape::Roundrect,
                        position!(-1.65, 3.41, angle),
                        size!(0.9, 1.2),
                        0.3.mm(),
                    ),
                    Pad::new(
                        "2",
                        PadType::ThruHole,
                        PadShape::Circle,
                        position!(-2.54, -5.08, angle),
                        size!(2.2, 2.2),
                        1.5.mm(),
                    ),
                    Pad::new(
                        "2",
                        PadType::ThruHole,
                        PadShape::Circle,
                        position!(-3.81, -2.54, angle),
                        size!(2.2, 2.2),
                        1.5.mm(),
                    ),
                    Pad::new(
                        "3",
                        PadType::ThruHole,
                        PadShape::Circle,
                        position!(2.54, -5.08, angle),
                        size!(2.2, 2.2),
                        1.5.mm(),
                    ),
                    Pad::new(
                        "3",
                        PadType::ThruHole,
                        PadShape::Circle,
                        position!(3.81, -2.54, angle),
                        size!(2.2, 2.2),
                        1.5.mm(),
                    ),
                    Pad::new(
                        "3",
                        PadType::ThruHole,
                        PadShape::Roundrect,
                        position!(1.65, 3.41, angle),
                        size!(0.9, 1.2),
                        0.3.mm(),
                    ),
                ],
            },
        )
    }
}
