use crate::{
    footprints::{Attribute, Footprint, FootprintSettings, Pad, PadShape, PadType, Property},
    kicad_pcb::Net,
    lines_front_back, point, position,
    primitives::{Position, Uuid},
    size,
    unit::IntoUnit,
};

/// An FFC connector.
pub struct FfcConnector {
    reference: String,
    position: Position,
    nets: [Net; 12],
}

impl FfcConnector {
    /// Creates a new FFC connector at the given position.
    pub fn new(reference: String, position: Position, nets: [Net; 12]) -> Self {
        Self {
            reference,
            position,
            nets,
        }
    }

    #[allow(clippy::too_many_lines, clippy::similar_names)]
    fn pads(self) -> Vec<Pad> {
        let angle = self.position.angle();
        let via_drill_diameter = 0.3.mm();

        let [net1, net2, net3, net4, net5, net6, net7, net8, net9, net10, net11, net12] = self.nets;

        vec![
            Pad::new(
                "1",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-5.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net1),
            ),
            Pad::new(
                "2",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-4.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net2),
            ),
            Pad::new(
                "3",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-3.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net3),
            ),
            Pad::new(
                "4",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-2.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net4),
            ),
            Pad::new(
                "5",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-1.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net5),
            ),
            Pad::new(
                "6",
                PadType::ThruHole,
                PadShape::Oval,
                position!(-0.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net6),
            ),
            Pad::new(
                "7",
                PadType::ThruHole,
                PadShape::Oval,
                position!(0.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net7),
            ),
            Pad::new(
                "8",
                PadType::ThruHole,
                PadShape::Oval,
                position!(1.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net8),
            ),
            Pad::new(
                "9",
                PadType::ThruHole,
                PadShape::Oval,
                position!(2.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net9),
            ),
            Pad::new(
                "10",
                PadType::ThruHole,
                PadShape::Oval,
                position!(3.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net10),
            ),
            Pad::new(
                "11",
                PadType::ThruHole,
                PadShape::Oval,
                position!(4.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net11),
            ),
            Pad::new(
                "12",
                PadType::ThruHole,
                PadShape::Oval,
                position!(5.5, 0, angle),
                size!(0.6, 1.8),
                via_drill_diameter,
                Some(net12),
            ),
            Pad::new(
                "13",
                PadType::ThruHole,
                PadShape::Roundrect,
                position!(-8.05, 2.17, angle),
                size!(2.4, 2.8),
                via_drill_diameter,
                None,
            ),
            Pad::new(
                "14",
                PadType::ThruHole,
                PadShape::Roundrect,
                position!(8.05, 2.17, angle),
                size!(2.4, 2.8),
                via_drill_diameter,
                None,
            ),
        ]
    }
}

impl From<FfcConnector> for Footprint {
    fn from(ffc_connector: FfcConnector) -> Self {
        let angle = ffc_connector.position.angle();

        Self(
            "ffc_connector_dual_side",
            FootprintSettings {
                layer: "F.Cu",
                uuid: Uuid::new(),
                at: ffc_connector.position,
                descr: "FFC connector dual side",
                tags: "FFC connector dual side",
                properties: vec![
                    Property::new(
                        "Reference",
                        ffc_connector.reference.clone(),
                        position!(0, 4.0, angle),
                    ),
                    Property::new(
                        "Value",
                        "FFC_Connector".to_owned(),
                        position!(0, 7.5, angle),
                    ),
                    Property::new("Footprint", String::new(), position!(0, 0, angle)),
                    Property::new("Datasheet", String::new(), position!(0, 0, angle)),
                    Property::new("Description", String::new(), position!(0, 0, angle)),
                ],
                attr: Attribute::ThroughHole,
                fp_lines: lines_front_back![
                    (point!(-9.8, -1.2), point!(-9.8, 6.6), "CrtYd"),
                    (point!(-9.8, 6.6), point!(9.8, 6.6), "CrtYd"),
                    (point!(9.8, -1.2), point!(-9.8, -1.2), "CrtYd"),
                    (point!(9.8, -1.2), point!(9.8, 6.6), "CrtYd"),
                    (point!(-9.5, 0.52), point!(-6.2, 0.52), "Fab"),
                    (point!(-9.5, 6.3), point!(-9.5, 0.52), "Fab"),
                    (point!(9.5, 0.52), point!(6.2, 0.52), "Fab"),
                    (point!(9.5, 6.3), point!(-9.5, 6.3), "Fab"),
                    (point!(9.5, 6.3), point!(9.5, 0.484), "Fab"),
                ],
                pads: ffc_connector.pads(),
            },
        )
    }
}
