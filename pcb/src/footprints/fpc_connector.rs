use crate::{
    footprints::{Attribute, Footprint, FootprintSettings, Pad, PadShape, PadType, Property},
    lines_front_back, point, position,
    primitives::{Position, Uuid},
    size,
    unit::IntoUnit,
};

/// An FPC connector.
pub struct FpcConnector {
    position: Position,
}

impl FpcConnector {
    /// Creates a new FPC connector at the given position.
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    #[allow(clippy::too_many_lines)]
    fn pads(&self) -> Vec<Pad> {
        let angle = self.position.angle();
        let via_drill_diameter = 0.3.mm();

        vec![
            Pad::new(
                "1",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-5.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "2",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-4.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "3",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-3.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "4",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-2.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "5",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-1.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "6",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-0.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "7",
                PadType::ThruHole,
                PadShape::Oval,
                position!(0.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "8",
                PadType::ThruHole,
                PadShape::Oval,
                position!(1.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "9",
                PadType::ThruHole,
                PadShape::Oval,
                position!(2.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "10",
                PadType::ThruHole,
                PadShape::Oval,
                position!(3.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "11",
                PadType::ThruHole,
                PadShape::Oval,
                position!(4.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "12",
                PadType::ThruHole,
                PadShape::Oval,
                position!(5.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
            ),
            Pad::new(
                "13",
                PadType::ThruHole,
                PadShape::Roundrect,
                position!(-8.05, 2.17, angle),
                size!(2.4, 2.8),
                via_drill_diameter,
            ),
            Pad::new(
                "14",
                PadType::ThruHole,
                PadShape::Roundrect,
                position!(8.05, 2.17, angle),
                size!(2.4, 2.8),
                via_drill_diameter,
            ),
        ]
    }
}

impl From<FpcConnector> for Footprint {
    fn from(fpc_connector: FpcConnector) -> Self {
        let angle = fpc_connector.position.angle();

        Self(
            "fpc_connector_dual_side",
            FootprintSettings {
                layer: "F.Cu",
                uuid: Uuid::new(),
                at: fpc_connector.position,
                descr: "FPC connector dual side",
                tags: "FPC connector dual side",
                properties: vec![
                    Property::new("Reference", "REF**", position!(0, 4.0, angle)),
                    Property::new("Value", "FPC_Connector", position!(0, 7.5, angle)),
                    Property::new("Footprint", "", position!(0, 0, angle)),
                    Property::new("Datasheet", "", position!(0, 0, angle)),
                    Property::new("Description", "", position!(0, 0, angle)),
                ],
                attr: Attribute::ThroughHole,
                fp_lines: lines_front_back![
                    (point!(-9.8, -1.2), point!(-9.8, 6.6), "CrtYd"),
                    (point!(-9.8, 6.6), point!(9.8, 6.6), "CrtYd"),
                    (point!(9.8, -1.2), point!(-9.8, -1.2), "CrtYd"),
                    (point!(9.8, -1.2), point!(9.8, 1.2), "CrtYd"),
                    (point!(-9.5, 0.52), point!(-6.2, 0.52), "Fab"),
                    (point!(-9.5, 6.3), point!(-9.5, 0.52), "Fab"),
                    (point!(9.5, 0.52), point!(6.2, 0.52), "Fab"),
                    (point!(9.5, 6.3), point!(-9.5, 6.3), "Fab"),
                    (point!(9.5, 6.3), point!(9.5, 0.484), "Fab"),
                ],
                pads: fpc_connector.pads(),
            },
        )
    }
}
