use std::f64::consts::PI;

use glam::{DAffine3, DVec3};

use crate::matrix_pcb::{
    segments::{Arc, BezierCurve},
    Segment, CLUSTER_CONNECTOR_ARC_RADIUS,
};

/// A connector between the finger and thumb cluster.
pub struct ClusterConnector {
    /// The Bézier curve segment in the center of the connector.
    pub bezier_curve: BezierCurve,
    /// The angle of the finger cluster arc.
    pub finger_cluster_arc_angle: f64,
    /// The angle of the thumb cluster arc.
    pub thumb_cluster_arc_angle: f64,
}

impl ClusterConnector {
    /// Creates a new cluster connector from the finger and thumb cluster anchor key positions.
    #[must_use]
    pub fn from_anchor_key_positions(_finger_key: DAffine3, _thumb_key: DAffine3) -> Self {
        todo!()
    }
}

impl Segment for ClusterConnector {
    fn positions(&self) -> Vec<DAffine3> {
        let bezier_positions = self.bezier_curve.positions();

        let finger_cluster_arc_position = *bezier_positions
            .first()
            .expect("there should always be a position")
            * DAffine3::from_rotation_z(PI);
        let thumb_cluster_arc_position = *bezier_positions
            .last()
            .expect("there should always be a position");

        let finger_cluster_arc = Arc::new(
            CLUSTER_CONNECTOR_ARC_RADIUS,
            self.finger_cluster_arc_angle,
            DVec3::NEG_Z,
        );
        let thumb_cluster_arc = Arc::new(
            CLUSTER_CONNECTOR_ARC_RADIUS,
            self.thumb_cluster_arc_angle,
            DVec3::NEG_Z,
        );

        finger_cluster_arc
            .positions()
            .iter()
            .rev()
            .map(|&position| finger_cluster_arc_position * position)
            .chain(bezier_positions)
            .chain(
                thumb_cluster_arc
                    .positions()
                    .iter()
                    .map(|&position| thumb_cluster_arc_position * position),
            )
            .collect()
    }

    fn length(&self) -> f64 {
        self.bezier_curve.length()
    }
}
