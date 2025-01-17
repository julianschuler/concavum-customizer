use crate::{
    footprints::{Attribute, Footprint, FootprintSettings, Pad, PadShape, PadType, Property},
    kicad_pcb::Net,
    lines_front_back, point, position,
    primitives::{Point, Position, Uuid},
    size,
    unit::Length,
};

/// The position of the row pad within a switch.
pub const ROW_PAD: Point = Point::new(Length::new(1.65), Length::new(3.4));
/// The position above the row pad within a switch.
pub const ABOVE_ROW_PAD: Point = Point::new(ROW_PAD.x(), Length::new(2.4));
/// The position below the row pad within a switch.
pub const BELOW_ROW_PAD: Point = Point::new(ROW_PAD.x(), Length::new(4.4));
/// The position left of the row pad within a switch.
pub const LEFT_OF_ROW_PAD: Point = Point::new(Length::new(0.0), ROW_PAD.y());
/// The position of the lower column pad within a switch.
pub const LOWER_COLUMN_PAD: Point = Point::new(Length::new(3.81), Length::new(-2.54));
/// The position of the upper column pad within a switch.
pub const UPPER_COLUMN_PAD: Point = Point::new(Length::new(2.54), Length::new(-5.08));

/// A Cherry MX switch.
pub struct Switch {
    reference: String,
    position: Position,
    column_net: Net,
    row_net: Net,
    internal_net: Net,
}

impl Switch {
    /// Creates a new switch at the given position using the given nets.
    pub fn new(
        reference: String,
        position: Position,
        row_net: Net,
        column_net: Net,
        internal_net: Net,
    ) -> Self {
        Self {
            reference,
            position,
            column_net,
            row_net,
            internal_net,
        }
    }

    fn pads(self) -> Vec<Pad> {
        let angle = self.position.angle();
        let via_drill_diameter = 0.3.into();
        let pin_drill_diameter = 1.5.into();

        vec![
            Pad::new(
                "",
                PadType::NpThruHole,
                PadShape::Circle,
                position!(0, 0, angle),
                size!(4, 4),
                4.into(),
                None,
            ),
            Pad::new(
                "1",
                PadType::ThruHole,
                PadShape::Roundrect,
                position!(ROW_PAD.x(), ROW_PAD.y(), angle),
                size!(0.9, 1.2),
                via_drill_diameter,
                Some(self.row_net),
            ),
            Pad::new(
                "2",
                PadType::ThruHole,
                PadShape::Circle,
                position!(UPPER_COLUMN_PAD.x(), UPPER_COLUMN_PAD.y(), angle),
                size!(2.2, 2.2),
                pin_drill_diameter,
                Some(self.column_net.clone()),
            ),
            Pad::new(
                "2",
                PadType::ThruHole,
                PadShape::Circle,
                position!(LOWER_COLUMN_PAD.x(), LOWER_COLUMN_PAD.y(), angle),
                size!(2.2, 2.2),
                pin_drill_diameter,
                Some(self.column_net),
            ),
            Pad::new(
                "3",
                PadType::ThruHole,
                PadShape::Circle,
                position!(-UPPER_COLUMN_PAD.x(), UPPER_COLUMN_PAD.y(), angle),
                size!(2.2, 2.2),
                pin_drill_diameter,
                Some(self.internal_net.clone()),
            ),
            Pad::new(
                "3",
                PadType::ThruHole,
                PadShape::Circle,
                position!(-LOWER_COLUMN_PAD.x(), LOWER_COLUMN_PAD.y(), angle),
                size!(2.2, 2.2),
                pin_drill_diameter,
                Some(self.internal_net.clone()),
            ),
            Pad::new(
                "3",
                PadType::ThruHole,
                PadShape::Roundrect,
                position!(-ROW_PAD.x(), ROW_PAD.y(), angle),
                size!(0.9, 1.2),
                via_drill_diameter,
                Some(self.internal_net),
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
                    Property::new(
                        "Reference",
                        switch.reference.clone(),
                        position!(0, -7.1, angle),
                    ),
                    Property::new("Value", "SW_Cherry_MX".to_owned(), position!(0, 7.1, angle)),
                    Property::new("Footprint", String::new(), position!(0, 0, angle)),
                    Property::new("Datasheet", String::new(), position!(0, 0, angle)),
                    Property::new("Description", String::new(), position!(0, 0, angle)),
                ],
                attr: Attribute::ThroughHole,
                fp_lines: lines_front_back![
                    (point!(2.25, 2.4), point!(2.25, 4.4), "SilkS"),
                    (point!(2.25, 2.4), point!(-1.65, 2.4), "SilkS"),
                    (point!(2.25, 4.4), point!(-1.65, 4.4), "SilkS"),
                    (point!(1.05, 2.4), point!(1.05, 4.4), "SilkS"),
                    (point!(6.6, -6.6), point!(-6.6, -6.6), "CrtYd"),
                    (point!(6.6, 6.6), point!(6.6, -6.6), "CrtYd"),
                    (point!(-6.6, -6.6), point!(-6.6, 6.6), "CrtYd"),
                    (point!(-6.6, 6.6), point!(6.6, 6.6), "CrtYd"),
                    (point!(1.4, 2.5), point!(1.4, 2.5), "Fab"),
                    (point!(1.4, 4.3), point!(1.4, 2.5), "Fab"),
                    (point!(0.75, 3.4), point!(0.35, 3.4), "Fab"),
                    (point!(0.35, 3.4), point!(0.35, 2.85), "Fab"),
                    (point!(0.35, 3.4), point!(0.35, 3.95), "Fab"),
                    (point!(0.35, 3.4), point!(-0.25, 3.0), "Fab"),
                    (point!(-0.25, 3.0), point!(-0.25, 3.8), "Fab"),
                    (point!(-0.25, 3.0), point!(-0.25, 3.8), "Fab"),
                    (point!(-0.25, 3.4), point!(-0.75, 3.4), "Fab"),
                    (point!(-0.25, 3.8), point!(0.35, 3.4), "Fab"),
                    (point!(-1.4, 2.5), point!(-1.4, 4.3), "Fab"),
                    (point!(-1.4, 4.3), point!(1.4, 4.3), "Fab"),
                    (point!(6.35, -6.35), point!(-6.35, -6.35), "Fab"),
                    (point!(6.35, 6.35), point!(6.35, -6.35), "Fab"),
                    (point!(-6.35, -6.35), point!(-6.35, 6.35), "Fab"),
                    (point!(-6.35, 6.35), point!(6.35, 6.35), "Fab"),
                ],
                fp_texts: Vec::new(),
                pads: switch.pads(),
            },
        )
    }
}
