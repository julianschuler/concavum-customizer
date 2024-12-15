use std::f64::consts::PI;

use glam::{dvec3, DAffine3, DMat3, DVec2, DVec3, Vec3Swizzles};

use crate::{
    geometry::{rotate_90_degrees, Ellipse, Line, Plane},
    matrix_pcb::{
        pad_center,
        segments::{Arc, BezierCurve},
        Segment, CLUSTER_CONNECTOR_ARC_RADIUS, CONNECTOR_WIDTH, FFC_PAD_OFFSET, FFC_PAD_SIZE,
        PAD_SIZE,
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
        // The plane normal is given as the average normal vector of the arc planes
        let plane_normal = (finger_key.z_axis + thumb_key.z_axis).normalize();
        let x_axis = DVec3::Y.cross(plane_normal);
        let y_axis = plane_normal.cross(x_axis);

        // Express arc center points in plane coordinate system (XY-plane)
        let plane_to_world = DMat3 {
            x_axis,
            y_axis,
            z_axis: plane_normal,
        };
        let world_to_plane = plane_to_world.inverse();

        let thumb_arc_center = (world_to_plane * arc_center(thumb_key, SideY::Top)).xy();
        let finger_arc_center = (world_to_plane * arc_center(finger_key, SideY::Bottom)).xy();

        // Extend arcs to circles and project them to the XY-plane
        let finger_ellipse =
            calculate_circle_projection(world_to_plane * finger_key.z_axis, finger_arc_center);
        let thumb_ellipse =
            calculate_circle_projection(world_to_plane * thumb_key.z_axis, thumb_arc_center);

        // Calculate the tangets between the ellipses and calculate the direction from it
        let tangents = finger_ellipse.tangents_to(&thumb_ellipse);
        let tangent = select_tangent(tangents, finger_arc_center, thumb_arc_center);
        let direction = plane_to_world * dvec3(tangent.y, -tangent.x, 0.0);

        // Calculate the arc angles and Bézier curve from the segment direction
        let finger_cluster_arc_angle = calculate_arc_angle(finger_key, direction, plane_normal);
        let thumb_cluster_arc_angle = calculate_arc_angle(thumb_key, direction, plane_normal);

        let start = arc_end(finger_cluster_arc_angle, finger_key, SideY::Bottom);
        let end = arc_end(thumb_cluster_arc_angle, thumb_key, SideY::Top);
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
/// Returns the start point of the arc for the given key position and side.
fn arc_start(position: DAffine3, side: SideY) -> DVec3 {
    let offset = match side {
        SideY::Bottom => (-FFC_PAD_OFFSET - FFC_PAD_SIZE.y / 2.0) * position.y_axis,
        SideY::Top => {
            (PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0 * position.x_axis
                + PAD_SIZE.y / 2.0 * position.y_axis
        }
    };

    pad_center(position) + offset
}

/// Returns the center point of the arc starting at the given key position and side.
fn arc_center(position: DAffine3, side: SideY) -> DVec3 {
    arc_start(position, side) + side.direction() * CLUSTER_CONNECTOR_ARC_RADIUS * position.x_axis
}

/// Returns the end position of the arc with the given angle starting at the given key position and side.
fn arc_end(angle: f64, position: DAffine3, side: SideY) -> DAffine3 {
    let translation = side.direction() * CLUSTER_CONNECTOR_ARC_RADIUS * DVec3::X;

    DAffine3 {
        matrix3: position.matrix3,
        translation: arc_start(position, side),
    } * DAffine3 {
        matrix3: DMat3::from_rotation_z(-angle),
        translation,
    } * DAffine3::from_translation(-translation)
}

/// Calculates the projection of a circle on a plane with the given normal axis
/// to the XY-plane, placing its projected center to the given position.
fn calculate_circle_projection(normal: DVec3, center: DVec2) -> Ellipse {
    let minor_axis = normal.xy().try_normalize().unwrap_or(DVec2::X);

    Ellipse::new(
        minor_axis,
        center,
        normal.z * CLUSTER_CONNECTOR_ARC_RADIUS,
        CLUSTER_CONNECTOR_ARC_RADIUS,
    )
}

/// Selects the tangent along the line segment connecting the finger and thumb cluster arcs.
fn select_tangent(
    tangents: [DVec3; 4],
    finger_arc_center: DVec2,
    thumb_arc_center: DVec2,
) -> DVec3 {
    let direction = thumb_arc_center - finger_arc_center;
    let homogenous_finger_arc_center = finger_arc_center.extend(1.0);
    let homogenous_thumb_arc_center = thumb_arc_center.extend(1.0);

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

/// Calculates the angle of the arc corresponding to the given position using the given direction
/// and projection plane normal vector.
fn calculate_arc_angle(position: DAffine3, direction: DVec3, plane_normal: DVec3) -> f64 {
    let plane = Plane::new(DVec3::ZERO, position.z_axis);
    let line = Line::new(direction, plane_normal);

    let projected_direction = plane
        .intersection(&line)
        .expect("there should always be an intersection");

    position.y_axis.angle_between(projected_direction)
}
