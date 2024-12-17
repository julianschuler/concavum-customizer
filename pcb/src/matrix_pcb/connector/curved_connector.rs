use model::matrix_pcb::{
    ClusterConnector, NormalColumnConnector, Segment, CLUSTER_CONNECTOR_ARC_RADIUS,
    CONNECTOR_WIDTH, PAD_SIZE,
};

use crate::{
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{OUTLINE_LAYER, OUTLINE_WIDTH},
    position,
    primitives::{Point, Position},
    unit::{Angle, IntoUnit, Length},
};

/// A curved connector described by two arcs.
pub struct CurvedConnector {
    first_arc: Arc,
    second_arc: Arc,
    end_switch_position: Position,
}

impl CurvedConnector {
    /// Creates a connector from a normal column connector and a switch position.
    pub fn from_normal_column_connector(
        normal_column_connector: &NormalColumnConnector,
        switch_position: Position,
    ) -> Self {
        let length = normal_column_connector.bezier_curve.length();
        let direction = normal_column_connector.left_arc_side.direction();

        let radius = (direction * normal_column_connector.arc_radius).mm();
        let angle = (direction * 90.0).deg();
        let offset = position!(
            PAD_SIZE.x / 2.0,
            -direction * (PAD_SIZE.y - CONNECTOR_WIDTH) / 2.0,
            None
        );

        let first_arc = Arc::new(switch_position + offset, -radius, -angle);
        let second_arc_start = first_arc.end_position() + position!(length, 0, None);
        let second_arc = Arc::new(second_arc_start, radius, angle);

        let end_switch_position = second_arc.end_position() + offset;

        Self {
            first_arc,
            second_arc,
            end_switch_position,
        }
    }

    /// Creates a connector from a cluster connector and a start position.
    pub fn from_cluster_connector(cluster_connector: &ClusterConnector, start: Position) -> Self {
        let length = cluster_connector.bezier_curve.length();

        let first_arc = Arc::new(
            start,
            -CLUSTER_CONNECTOR_ARC_RADIUS.mm(),
            -cluster_connector.finger_cluster_arc_angle.rad(),
        );
        let second_arc_start = first_arc.end_position() + position!(length, 0, None);
        let second_arc = Arc::new(
            second_arc_start,
            CLUSTER_CONNECTOR_ARC_RADIUS.mm(),
            cluster_connector.thumb_cluster_arc_angle.rad(),
        );

        let end_switch_position = second_arc.end_position()
            + position!(
                PAD_SIZE.y / 2.0,
                (PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0,
                Some(90.deg())
            );

        Self {
            first_arc,
            second_arc,
            end_switch_position,
        }
    }

    /// Returns the start position of the connector
    pub fn start_position(&self) -> Position {
        self.first_arc.start
    }

    /// Returns the end position of the connector.
    pub fn end_position(&self) -> Position {
        self.second_arc.end_position()
    }

    /// Returns the position of the switch at the end of the connector
    pub fn end_switch_position(&self) -> Position {
        self.end_switch_position
    }

    /// Adds the outline of the connector to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        let offset = (CONNECTOR_WIDTH / 2.0).mm();

        let first_arc_top = self.first_arc.offset(offset);
        let first_arc_bottom = self.first_arc.offset(-offset);
        let second_arc_top = self.second_arc.offset(offset);
        let second_arc_bottom = self.second_arc.offset(-offset);

        pcb.add_graphical_line(
            first_arc_top.end(),
            second_arc_top.start(),
            OUTLINE_WIDTH,
            OUTLINE_LAYER,
        );
        pcb.add_graphical_line(
            first_arc_bottom.end(),
            second_arc_bottom.start(),
            OUTLINE_WIDTH,
            OUTLINE_LAYER,
        );
        for arc in [
            first_arc_top,
            first_arc_bottom,
            second_arc_top,
            second_arc_bottom,
        ] {
            pcb.add_graphical_arc(
                arc.start(),
                arc.mid(),
                arc.end(),
                OUTLINE_WIDTH,
                OUTLINE_LAYER,
            );
        }
    }

    /// Adds a track to the PCB with the given offset to the center.
    pub fn add_track(
        &self,
        pcb: &mut KicadPcb,
        offset: Length,
        width: Length,
        layer: &'static str,
        net: &Net,
    ) {
        let first_arc = self.first_arc.offset(offset);
        let second_arc = self.second_arc.offset(offset);

        pcb.add_segment(first_arc.end(), second_arc.start(), width, layer, net);
        for arc in [first_arc, second_arc] {
            pcb.add_arc(arc.start(), arc.mid(), arc.end(), width, layer, net);
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
        let start = self.start + Position::new(0.mm(), offset, None);
        let radius = self.radius + offset;

        Self {
            start,
            radius,
            angle: self.angle,
        }
    }

    /// Returns the position on the arc with the angle from start.
    fn position(&self, angle: Angle) -> Position {
        self.start
            + Position::new(0.mm(), -self.radius, Some(angle))
            + Position::new(0.mm(), self.radius, None)
    }
}
