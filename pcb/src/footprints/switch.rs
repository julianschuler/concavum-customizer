use crate::{
    footprints::{Attribute, Footprint, FootprintSettings, Pad, PadShape, PadType, Property},
    lines_front_back, point, position,
    primitives::{Position, Uuid},
    size,
    unit::IntoUnit,
};

/// A Cherry MX switch.
pub struct Switch {
    position: Position,
}

impl Switch {
    /// Creates a new switch at the given position.
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    fn pads(&self) -> Vec<Pad> {
        let angle = self.position.angle();
        let via_drill_diameter = 0.3.mm();
        let pin_drill_diameter = 1.5.mm();

        vec![
            Pad::new(
                "",
                PadType::NpThruHole,
                PadShape::Circle,
                position!(0, 0, angle),
                size!(4, 4),
                4.mm(),
            ),
            Pad::new(
                "1",
                PadType::ThruHole,
                PadShape::Roundrect,
                position!(-1.65, 3.41, angle),
                size!(0.9, 1.2),
                via_drill_diameter,
            ),
            Pad::new(
                "2",
                PadType::ThruHole,
                PadShape::Circle,
                position!(-2.54, -5.08, angle),
                size!(2.2, 2.2),
                pin_drill_diameter,
            ),
            Pad::new(
                "2",
                PadType::ThruHole,
                PadShape::Circle,
                position!(-3.81, -2.54, angle),
                size!(2.2, 2.2),
                pin_drill_diameter,
            ),
            Pad::new(
                "3",
                PadType::ThruHole,
                PadShape::Circle,
                position!(2.54, -5.08, angle),
                size!(2.2, 2.2),
                pin_drill_diameter,
            ),
            Pad::new(
                "3",
                PadType::ThruHole,
                PadShape::Circle,
                position!(3.81, -2.54, angle),
                size!(2.2, 2.2),
                pin_drill_diameter,
            ),
            Pad::new(
                "3",
                PadType::ThruHole,
                PadShape::Roundrect,
                position!(1.65, 3.41, angle),
                size!(0.9, 1.2),
                via_drill_diameter,
            ),
        ]
    }
}

impl From<Switch> for Footprint {
    fn from(switch: Switch) -> Self {
        let angle = switch.position.angle();

        Self(
            "switch_dual_side",
            FootprintSettings {
                layer: "F.Cu",
                uuid: Uuid::new(),
                at: switch.position,
                descr: "Cherry MX switch with diode dual side",
                tags: "Cherry MX switch with diode dual side",
                properties: vec![
                    Property::new("Reference", "REF**", position!(0, -7.5, angle)),
                    Property::new("Value", "SW_Cherry_MX", position!(0, 7.5, angle)),
                    Property::new("Footprint", "", position!(0, 0, angle)),
                    Property::new("Datasheet", "", position!(0, 0, angle)),
                    Property::new("Description", "", position!(0, 0, angle)),
                ],
                attr: Attribute::ThroughHole,
                fp_lines: lines_front_back![
                    (point!(-2.25, 2.41), point!(-2.25, 4.41), "SilkS"),
                    (point!(-2.25, 2.41), point!(1.65, 2.41), "SilkS"),
                    (point!(-2.25, 4.41), point!(1.65, 4.41), "SilkS"),
                    (point!(-1.05, 2.41), point!(-1.05, 4.41), "SilkS"),
                    (point!(-6.6, -6.6), point!(6.6, -6.6), "CrtYd"),
                    (point!(-6.6, 6.6), point!(-6.6, -6.6), "CrtYd"),
                    (point!(6.6, -6.6), point!(6.6, 6.6), "CrtYd"),
                    (point!(6.6, 6.6), point!(-6.6, 6.6), "CrtYd"),
                    (point!(-1.4, 2.51), point!(-1.4, 2.51), "Fab"),
                    (point!(-1.4, 4.31), point!(-1.4, 2.51), "Fab"),
                    (point!(-0.75, 3.41), point!(-0.35, 3.41), "Fab"),
                    (point!(-0.35, 3.41), point!(-0.35, 2.86), "Fab"),
                    (point!(-0.35, 3.41), point!(-0.35, 3.96), "Fab"),
                    (point!(-0.35, 3.41), point!(0.25, 3.01), "Fab"),
                    (point!(0.25, 3.01), point!(0.25, 3.81), "Fab"),
                    (point!(0.25, 3.01), point!(0.25, 3.81), "Fab"),
                    (point!(0.25, 3.41), point!(0.75, 3.41), "Fab"),
                    (point!(0.25, 3.81), point!(-0.35, 3.41), "Fab"),
                    (point!(1.4, 2.51), point!(1.4, 4.31), "Fab"),
                    (point!(1.4, 4.31), point!(-1.4, 4.31), "Fab"),
                    (point!(-6.35, -6.35), point!(6.35, -6.35), "Fab"),
                    (point!(-6.35, 6.35), point!(-6.35, -6.35), "Fab"),
                    (point!(6.35, -6.35), point!(6.35, 6.35), "Fab"),
                    (point!(6.35, 6.35), point!(-6.35, 6.35), "Fab"),
                ],
                pads: switch.pads(),
            },
        )
    }
}
