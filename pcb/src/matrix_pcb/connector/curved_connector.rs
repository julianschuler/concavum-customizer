use model::matrix_pcb::{
    ClusterConnector, NormalColumnConnector, Segment, ARC_RADIUS, CONNECTOR_WIDTH,
    MINIMUM_SEGMENT_LENGTH, PAD_SIZE,
};

use crate::{
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{connector::AttachmentSide, OUTLINE_LAYER, OUTLINE_WIDTH, TRACK_WIDTH},
    point, position,
    primitives::{Point, Position},
    unit::{Angle, IntoAngle, Length},
};

/// A curved connector described by two arcs and straight segments.
pub struct CurvedConnector {
    first_arc: Arc,
    second_arc: Arc,
    segment_length: Length,
    start_switch_position: Position,
    end_switch_position: Position,
    attachment_side: AttachmentSide,
}

impl CurvedConnector {
    /// Creates a connector from a normal column connector and a switch position.
    pub fn from_normal_column_connector(
        normal_column_connector: &NormalColumnConnector,
        start_switch_position: Position,
    ) -> Self {
        let length = normal_column_connector.bezier_curve.length();
        let direction = normal_column_connector.left_arc_side.direction();
        let segment_length = normal_column_connector.segment_length.into();

        let radius = Length::from(direction * ARC_RADIUS);
        let angle = (direction * 90.0).deg();
        let offset = position!(
            PAD_SIZE.x / 2.0 + normal_column_connector.segment_length,
            -direction * (PAD_SIZE.y - CONNECTOR_WIDTH) / 2.0,
            None
        );

        let first_arc = Arc::new(start_switch_position + offset, -radius, -angle);
        let second_arc_start = first_arc.end_position() + position!(length, 0, None);
        let second_arc = Arc::new(second_arc_start, radius, angle);

        let end_switch_position = second_arc.end_position() + offset;

        #[allow(clippy::float_cmp)]
        let attachment_side = if direction == 1.0 {
            AttachmentSide::Bottom
        } else {
            AttachmentSide::Top
        };

        Self {
            first_arc,
            second_arc,
            segment_length,
            start_switch_position,
            end_switch_position,
            attachment_side,
        }
    }

    /// Creates a connector from a cluster connector and a start position.
    pub fn from_cluster_connector(
        cluster_connector: &ClusterConnector,
        start: Position,
        start_switch_position: Position,
    ) -> Self {
        let segment_length = MINIMUM_SEGMENT_LENGTH.into();
        let length = cluster_connector.bezier_curve.length();

        let first_arc = Arc::new(
            start + position!(MINIMUM_SEGMENT_LENGTH, 0, None),
            (-ARC_RADIUS).into(),
            -cluster_connector.finger_cluster_arc_angle.rad(),
        );
        let second_arc_start = first_arc.end_position() + position!(length, 0, None);
        let second_arc = Arc::new(
            second_arc_start,
            ARC_RADIUS.into(),
            cluster_connector.thumb_cluster_arc_angle.rad(),
        );

        let end_switch_position = second_arc.end_position()
            + position!(
                PAD_SIZE.y / 2.0 + MINIMUM_SEGMENT_LENGTH,
                (PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0,
                Some(90.deg())
            );

        let attachment_side = AttachmentSide::Center;

        Self {
            first_arc,
            second_arc,
            segment_length,
            start_switch_position,
            end_switch_position,
            attachment_side,
        }
    }

    /// Returns the start position of the connector
    pub fn start_position(&self) -> Position {
        self.first_arc.start - position!(self.segment_length, 0, None)
    }

    /// Returns the position of the switch at the start of the connector
    pub fn start_switch_position(&self) -> Position {
        self.start_switch_position
    }

    /// Returns the end position of the connector.
    pub fn end_position(&self) -> Position {
        self.second_arc.end_position() + position!(self.segment_length, 0, None)
    }

    /// Returns the position of the switch at the end of the connector
    pub fn end_switch_position(&self) -> Position {
        self.end_switch_position
    }

    /// Returns the attachment side of the end of the connector.
    pub fn end_attachment_side(&self) -> AttachmentSide {
        self.attachment_side
    }

    /// Adds the outline of the connector to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        let start_position = self.start_position();
        let end_position = self.end_position();

        for sign in [-1, 1] {
            let offset = sign * Length::from(CONNECTOR_WIDTH / 2.0);

            let first_arc = self.first_arc.offset(offset);
            let second_arc = self.second_arc.offset(offset);

            pcb.add_graphical_line(
                first_arc.end(),
                second_arc.start(),
                OUTLINE_WIDTH,
                OUTLINE_LAYER,
            );

            for arc in [first_arc, second_arc] {
                pcb.add_graphical_arc(
                    arc.start(),
                    arc.mid(),
                    arc.end(),
                    OUTLINE_WIDTH,
                    OUTLINE_LAYER,
                );
            }

            for (sign, position) in [(1, start_position), (-1, end_position)] {
                pcb.add_graphical_line(
                    position + point!(0, offset),
                    position + point!(sign * self.segment_length, offset),
                    OUTLINE_WIDTH,
                    OUTLINE_LAYER,
                );
            }
        }
    }

    /// Adds a track to the PCB with the given offset to the center.
    pub fn add_track(&self, pcb: &mut KicadPcb, offset: Length, layer: &'static str, net: &Net) {
        let first_arc = self.first_arc.offset(offset);
        let second_arc = self.second_arc.offset(offset);

        pcb.add_segment(first_arc.end(), second_arc.start(), TRACK_WIDTH, layer, net);
        for arc in [first_arc, second_arc] {
            pcb.add_arc(arc.start(), arc.mid(), arc.end(), TRACK_WIDTH, layer, net);
        }

        for (sign, position) in [(1, self.start_position()), (-1, self.end_position())] {
            pcb.add_segment(
                position + point!(0, offset),
                position + point!(sign * self.segment_length, offset),
                TRACK_WIDTH,
                layer,
                net,
            );
        }
    }
}

/// An arc tangential to a start position.
struct Arc {
    start: Position,
    radius: Length,
    angle: Angle,
}

impl Arc {
    /// Creates a new arc from the given start position, radius and angle.
    pub fn new(start: Position, radius: Length, angle: Angle) -> Self {
        Self {
            start,
            radius,
            angle,
        }
    }

    /// Returns the start point of the arc.
    pub fn start(&self) -> Point {
        self.start.point()
    }

    /// Returns the mid point of the arc.
    pub fn mid(&self) -> Point {
        self.position(self.angle / 2).point()
    }

    /// Returns the end point of the arc.
    pub fn end(&self) -> Point {
        self.end_position().point()
    }

    /// Returns the end position of the arc.
    pub fn end_position(&self) -> Position {
        self.position(self.angle)
    }

    /// Offsets the arc by the given length.
    pub fn offset(&self, offset: Length) -> Self {
        let start = self.start + position!(0, offset, None);
        let radius = self.radius + offset;

        Self {
            start,
            radius,
            angle: self.angle,
        }
    }

    /// Returns the position on the arc with the angle from start.
    fn position(&self, angle: Angle) -> Position {
        self.start + position!(0, -self.radius, Some(angle)) + position!(0, self.radius, None)
    }
}
