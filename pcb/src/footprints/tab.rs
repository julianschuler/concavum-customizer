use crate::{
    footprints::{Attribute, Footprint, FootprintSettings, Line, Property, Text},
    point, position,
    primitives::{Position, Uuid},
    unit::{IntoAngle, Length},
};

/// A marker for placing a tab.
pub struct Tab {
    position: Position,
}

impl Tab {
    /// The width of the tab.
    pub const WIDTH: Length = Length::new(10.0);

    /// Creates a new tab marker at the given position.
    pub fn new(position: Position) -> Self {
        Self { position }
    }
}

impl From<Tab> for Footprint {
    fn from(tab: Tab) -> Self {
        let angle = tab.position.angle();

        Self(
            "kikit:Tab",
            FootprintSettings {
                layer: "F.Cu",
                uuid: Uuid::new(),
                at: tab.position,
                descr: "A marker for placing a tab",
                tags: "",
                properties: vec![
                    Property::new("Reference", String::new(), position!(0, 0, angle)),
                    Property::new("Value", "Tab".to_owned(), position!(-3, -1, angle)),
                    Property::new("Footprint", String::new(), position!(0, 0, angle)),
                    Property::new("Datasheet", String::new(), position!(0, 0, angle)),
                    Property::new("Description", String::new(), position!(0, 0, angle)),
                ],
                attr: Attribute::Marker,
                fp_lines: vec![
                    Line::new(point!(0, 0), point!(-3, 0), "Dwgs.User"),
                    Line::new(point!(0, 0), point!(-1, -1), "Dwgs.User"),
                    Line::new(point!(0, 0), point!(-1, 1), "Dwgs.User"),
                    Line::new(point!(0, 1), point!(0, -1), "Dwgs.User"),
                ],
                fp_texts: vec![Text::new(
                    "KIKIT: width: 10mm",
                    position!(-5.5, 0, Some(90.deg() + angle.unwrap_or_default())),
                )],
                pads: Vec::new(),
            },
        )
    }
}
