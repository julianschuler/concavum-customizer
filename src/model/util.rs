use fidget::{context::Node, Context};
use glam::{dvec3, DAffine3, DQuat, DVec2, DVec3, Vec3Swizzles};

use crate::model::{
    geometry::{zvec, Plane},
    primitives::{Csg, Result, SimplePolygon, Transforms},
};

/// Upper bound for the size of a mount
pub struct MountSize {
    pub width: f64,
    pub length: f64,
    pub height: f64,
}

impl MountSize {
    pub fn from_positions<'a>(
        positions: impl IntoIterator<Item = &'a DAffine3>,
        key_clearance: &DVec2,
        circumference_distance: f64,
    ) -> Self {
        let padding = key_clearance.x + key_clearance.y;

        let (min, max) = positions.into_iter().fold(
            (
                dvec3(f64::INFINITY, f64::INFINITY, f64::INFINITY),
                dvec3(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            ),
            |(min, max), point| (min.min(point.translation), max.max(point.translation)),
        );

        let width = max.x - min.x + 2.0 * circumference_distance + padding;
        let length = max.y - min.y + 2.0 * circumference_distance + padding;
        let height = max.z + padding;

        Self {
            width,
            length,
            height,
        }
    }
}

#[derive(Clone, Copy)]
pub enum SideX {
    Left,
    Right,
}

impl SideX {
    pub fn direction(self) -> f64 {
        match self {
            SideX::Left => -1.0,
            SideX::Right => 1.0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum SideY {
    Bottom,
    Top,
}

impl SideY {
    pub fn direction(self) -> f64 {
        match self {
            SideY::Bottom => -1.0,
            SideY::Top => 1.0,
        }
    }
}

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

pub fn corner_point(
    position: &DAffine3,
    side_x: SideX,
    side_y: SideY,
    key_clearance: &DVec2,
) -> DVec3 {
    position.translation
        + side_x.direction() * key_clearance.x * position.x_axis
        + side_y.direction() * key_clearance.y * position.y_axis
}

pub fn side_point(position: &DAffine3, side: Side, key_clearance: &DVec2) -> DVec3 {
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
    context: &mut Context,
    points: impl IntoIterator<Item = DVec3>,
    plane: &Plane,
    height: f64,
) -> Result<Node> {
    let rotation = DQuat::from_rotation_arc(plane.normal(), DVec3::Z);
    let offset = (rotation * plane.point()).z;

    let vertices = points
        .into_iter()
        .map(|point| (rotation * point).xy())
        .collect();

    let polygon = SimplePolygon::new(vertices);
    let prism = context.extrude(polygon, offset, offset + height)?;

    let affine = DAffine3::from_quat(rotation.inverse());
    context.affine(prism, affine)
}

/// Creates a sheared prism by projecting points to a plane, extruding to a
/// given height and sheering it to the given direction.
///
/// The points must be in a counter-clockwise order.
pub fn sheared_prism_from_projected_points(
    context: &mut Context,
    points: impl IntoIterator<Item = DVec3>,
    plane: &Plane,
    height: f64,
    direction: DVec3,
) -> Result<Node> {
    let rotation = DQuat::from_rotation_arc(plane.normal(), DVec3::Z);
    let offset = (rotation * plane.point()).z;
    let shearing_direction = rotation * direction;

    let vertices = points
        .into_iter()
        .map(|point| (rotation * point).xy())
        .collect();

    let polygon = SimplePolygon::new(vertices);
    let prism = context.extrude(polygon, 0.0, height)?;
    let prism = context.shear(prism, shearing_direction.xy(), shearing_direction.z)?;

    let affine = DAffine3::from_translation(zvec(offset)) * DAffine3::from_quat(rotation.inverse());
    context.affine(prism, affine)
}
