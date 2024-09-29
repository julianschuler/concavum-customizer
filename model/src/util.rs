use fidget::context::Tree;
use glam::{DAffine3, DMat3, DQuat, DVec2, DVec3, Vec3Swizzles};

use crate::{
    geometry::{vec_z, Plane},
    primitives::{Bounds, Csg, IntoTree, SimplePolygon, Transforms},
};

/// A side along the X-axis.
#[derive(Clone, Copy)]
pub enum SideX {
    Left,
    Right,
}

impl SideX {
    /// Returns the sign of the direction of the side along the X-axis.
    pub fn direction(self) -> f64 {
        match self {
            SideX::Left => -1.0,
            SideX::Right => 1.0,
        }
    }
}

/// A side along the Y-axis.
#[derive(Clone, Copy)]
pub enum SideY {
    Bottom,
    Top,
}

impl SideY {
    /// Returns the sign of the direction of the side along Y-axis.
    pub fn direction(self) -> f64 {
        match self {
            SideY::Bottom => -1.0,
            SideY::Top => 1.0,
        }
    }

    /// Returns the opposite side.
    pub fn opposite(self) -> Self {
        match self {
            SideY::Bottom => SideY::Top,
            SideY::Top => SideY::Bottom,
        }
    }
}

/// A side along the X- or Y-axis.
#[derive(Clone, Copy)]
pub enum Side {
    Left,
    Right,
    Bottom,
    Top,
}

impl From<SideX> for Side {
    fn from(side_x: SideX) -> Self {
        match side_x {
            SideX::Left => Self::Left,
            SideX::Right => Self::Right,
        }
    }
}

impl From<SideY> for Side {
    fn from(side_y: SideY) -> Self {
        match side_y {
            SideY::Bottom => Self::Bottom,
            SideY::Top => Self::Top,
        }
    }
}

/// Creates a point in a corner given by two sides (e.g. Bottom and Left) and a clearance.
pub fn corner_point(
    position: DAffine3,
    side_x: SideX,
    side_y: SideY,
    key_clearance: DVec2,
) -> DVec3 {
    position.translation
        + side_x.direction() * key_clearance.x * position.x_axis
        + side_y.direction() * key_clearance.y * position.y_axis
}

/// Creates a point at the given side (e.g. Left) for the given clearance.
pub fn side_point(position: DAffine3, side: Side, key_clearance: DVec2) -> DVec3 {
    match side {
        Side::Left => position.translation - key_clearance.x * position.x_axis,
        Side::Right => position.translation + key_clearance.x * position.x_axis,
        Side::Bottom => position.translation - key_clearance.y * position.y_axis,
        Side::Top => position.translation + key_clearance.y * position.y_axis,
    }
}

/// Creates a prism by projecting points to a plane and extruding to a given height.
///
/// The points must be in a counter-clockwise order.
pub fn prism_from_projected_points(
    points: impl IntoIterator<Item = DVec3>,
    plane: &Plane,
    height: f64,
) -> Tree {
    let rotation = DQuat::from_rotation_arc(plane.normal(), DVec3::Z);
    let offset = (rotation * plane.point()).z;

    let vertices = points
        .into_iter()
        .map(|point| (rotation * point).xy())
        .collect();

    let affine = DAffine3::from_quat(rotation.inverse());

    SimplePolygon::new(vertices)
        .into_tree()
        .extrude(offset, offset + height)
        .affine(affine)
}

/// Creates a sheared prism by projecting points to a plane, extruding to a
/// given height and shearing it to the given direction.
///
/// The points must be in a counter-clockwise order.
pub fn sheared_prism_from_projected_points(
    points: impl IntoIterator<Item = DVec3>,
    plane: &Plane,
    height: f64,
    direction: DVec3,
) -> Tree {
    let rotation = DQuat::from_rotation_arc(plane.normal(), DVec3::Z);
    let offset = (rotation * plane.point()).z;
    let shearing_direction = rotation * direction;

    let vertices = points
        .into_iter()
        .map(|point| (rotation * point).xy())
        .collect();

    let affine =
        DAffine3::from_quat(rotation.inverse()) * DAffine3::from_translation(vec_z(offset));

    SimplePolygon::new(vertices)
        .into_tree()
        .extrude(0.0, height)
        .shear(shearing_direction.xy(), shearing_direction.z)
        .affine(affine)
}

/// Creates bounds from outline points and height.
pub fn bounds_from_outline_points_and_height(
    outline_points: &[DVec2],
    height: f64,
    circumference_distance: f64,
) -> Bounds {
    let (min, max) = outline_points.iter().fold(
        (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)),
        |(min, max), point| (point.min(min), point.max(max)),
    );

    let padding = DVec2::splat(circumference_distance);
    let min = (min - padding).extend(0.0);
    let max = (max + padding).extend(height);

    Bounds { min, max }
}

/// Returns the unit vectors projected to a plane given by the normal.
/// The vectors are scaled such that it translates every point inside
/// the bound to the outside.
pub fn projected_unit_vectors(normal: DVec3, bounds: Bounds) -> DMat3 {
    let plane = Plane::new(DVec3::ZERO, normal);

    let x_axis = plane.project_vector(DVec3::X).normalize_or_zero();
    let y_axis = plane.project_vector(DVec3::Y).normalize_or_zero();
    let z_axis = plane.project_vector(DVec3::Z).normalize_or_zero();

    bounds.diameter()
        * DMat3 {
            x_axis,
            y_axis,
            z_axis,
        }
}
