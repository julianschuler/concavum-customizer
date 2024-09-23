use std::f64::consts::PI;

use glam::{dvec3, DAffine3, DMat3, DVec2, DVec3, Vec3Swizzles};

use crate::{
    geometry::{rotate_90_degrees, Ellipse, Plane},
    matrix_pcb::{
        pad_center,
        segments::{Arc, BezierCurve},
        Segment, CLUSTER_CONNECTOR_ARC_RADIUS, PAD_SIZE,
    },
    util::SideY,
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
    pub fn from_anchor_key_positions(finger_key: DAffine3, thumb_key: DAffine3) -> Self {
        // Create tranformations between world and projection plane in X and Y
        let z_axis = (finger_key.z_axis + thumb_key.z_axis).normalize();
        let x_axis = DVec3::Y.cross(z_axis);
        let y_axis = z_axis.cross(x_axis);

        let plane_to_world = DMat3 {
            x_axis,
            y_axis,
            z_axis,
        };
        let world_to_plane = plane_to_world.inverse();

        // Project arc axis and center to plane coordinates
        let finger_arc_axis = world_to_plane * finger_key.z_axis;
        let thumb_arc_axis = world_to_plane * thumb_key.z_axis;
        let finger_arc_center = (world_to_plane * arc_center(finger_key, SideY::Bottom)).xy();
        let thumb_arc_center = (world_to_plane * arc_center(thumb_key, SideY::Top)).xy();

        // Extend arcs to circles and project them to the XY-plane, creating ellipses
        // with the minor axes along the projected circle normal vectors
        let finger_ellipse_minor_axis = finger_arc_axis.xy().try_normalize().unwrap_or(DVec2::X);
        let thumb_ellipse_minor_axis = thumb_arc_axis.xy().try_normalize().unwrap_or(DVec2::X);
        let finger_ellipse = Ellipse::new(
            finger_ellipse_minor_axis,
            finger_arc_center,
            finger_arc_axis.z * CLUSTER_CONNECTOR_ARC_RADIUS,
            CLUSTER_CONNECTOR_ARC_RADIUS,
        );
        let thumb_ellipse = Ellipse::new(
            thumb_ellipse_minor_axis,
            thumb_arc_center,
            thumb_arc_axis.z * CLUSTER_CONNECTOR_ARC_RADIUS,
            CLUSTER_CONNECTOR_ARC_RADIUS,
        );

        // Calculate the tangents between the ellipses, select the correct and get its direction
        let tangents = finger_ellipse.tangents_to(&thumb_ellipse);
        let tangent = select_tangent(tangents, finger_arc_center, thumb_arc_center);
        let direction = plane_to_world * dvec3(tangent.y, -tangent.x, 0.0);

        // Project the direction of the tangent to the arc planes and calculate the arc angles from it
        let finger_arc_plane = Plane::new(pad_center(finger_key), finger_key.z_axis);
        let thumb_arc_plane = Plane::new(pad_center(thumb_key), thumb_key.z_axis);

        let direction_finger_plane = finger_arc_plane.project_vector(direction);
        let direction_thumb_plane = thumb_arc_plane.project_vector(direction);

        let finger_cluster_arc_angle = finger_key.y_axis.angle_between(direction_finger_plane);
        let thumb_cluster_arc_angle = thumb_key.y_axis.angle_between(direction_thumb_plane);

        // Calculate the start and end positions of the Bézier curve using the arc angles
        let start = arc_end(finger_key, finger_cluster_arc_angle, SideY::Bottom);
        let end = arc_end(thumb_key, thumb_cluster_arc_angle, SideY::Top);

        let bezier_curve = BezierCurve::from_positions(start, end);

        Self {
            bezier_curve,
            finger_cluster_arc_angle,
            thumb_cluster_arc_angle,
        }
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

/// Selects the tangent along the line segment connecting the finger and thumb cluster arcs.
fn select_tangent(
    tangents: [DVec3; 4],
    finger_arc_center: DVec2,
    thumb_arc_center: DVec2,
) -> DVec3 {
    let direction = thumb_arc_center - finger_arc_center;
    let homogenous_finger_arc_center = dvec3(finger_arc_center.x, finger_arc_center.y, 1.0);
    let homogenous_thumb_arc_center = dvec3(thumb_arc_center.x, thumb_arc_center.y, 1.0);

    tangents
        .into_iter()
        .map(|tangent| {
            let sign = rotate_90_degrees(tangent.xy()).dot(direction).signum();

            sign * tangent
        })
        .find(|&tangent| {
            tangent.dot(homogenous_thumb_arc_center) < 0.0
                && tangent.dot(homogenous_finger_arc_center) > 0.0
        })
        .expect("one of the tangents should match the predicate")
}

/// Returns the center point of the arc starting at the given key position and side.
fn arc_center(position: DAffine3, side: SideY) -> DVec3 {
    let direction = side.direction();

    pad_center(position)
        + direction * CLUSTER_CONNECTOR_ARC_RADIUS * position.x_axis
        + direction * PAD_SIZE.y / 2.0 * position.y_axis
}

/// Returns the end position of the arc with the given angle.
fn arc_end(position: DAffine3, angle: f64, side: SideY) -> DAffine3 {
    let direction = side.direction();
    let translation = direction * CLUSTER_CONNECTOR_ARC_RADIUS * DVec3::X;

    let arc_start_translation =
        pad_center(position) + direction * PAD_SIZE.y / 2.0 * position.y_axis;

    DAffine3 {
        matrix3: position.matrix3,
        translation: arc_start_translation,
    } * DAffine3 {
        matrix3: DMat3::from_rotation_z(-angle),
        translation,
    } * DAffine3::from_translation(-translation)
}
