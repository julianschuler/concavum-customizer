use glam::{Affine3A, Vec2, Vec3A};
use libfive::{Region3, TreeVec2, TreeVec3};
use opencascade::primitives::{Solid, Wire};

use crate::model::geometry::{Plane, Project};

#[inline]
pub fn vec2(x: f32, y: f32) -> TreeVec2 {
    TreeVec2::new(x, y)
}

#[inline]
pub fn vec3(x: f32, y: f32, z: f32) -> TreeVec3 {
    TreeVec3::new(x, y, z)
}

#[inline]
pub fn centered_cubic_region(size: f32) -> Region3 {
    Region3::new(-size, size, -size, size, -size, size)
}

/// Upper bound for the size of a mount
pub struct MountSize {
    pub width: f32,
    pub length: f32,
    pub height: f32,
}

impl MountSize {
    pub fn from_positions<'a>(
        positions: impl IntoIterator<Item = &'a Affine3A>,
        key_clearance: &Vec2,
        circumference_distance: f32,
    ) -> Self {
        let padding = key_clearance.x + key_clearance.y;

        let (min, max) = positions.into_iter().fold(
            (
                glam::vec3a(f32::INFINITY, f32::INFINITY, f32::INFINITY),
                glam::vec3a(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
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
    pub fn direction(self) -> f32 {
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
    pub fn direction(self) -> f32 {
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
    position: &Affine3A,
    side_x: SideX,
    side_y: SideY,
    key_clearance: &Vec2,
) -> Vec3A {
    (position.translation
        + side_x.direction() * key_clearance.x * position.x_axis
        + side_y.direction() * key_clearance.y * position.y_axis)
        .into()
}

pub fn side_point(position: &Affine3A, side: Side, key_clearance: &Vec2) -> Vec3A {
    match side {
        Side::Left => position.translation - key_clearance.x * position.x_axis,
        Side::Right => position.translation + key_clearance.x * position.x_axis,
        Side::Bottom => position.translation - key_clearance.y * position.y_axis,
        Side::Top => position.translation + key_clearance.y * position.y_axis,
    }
    .into()
}

pub fn wire_from_points(points: impl IntoIterator<Item = Vec3A>, plane: &Plane) -> Wire {
    let points = points
        .into_iter()
        .map(|point| point.project_to(plane).as_dvec3());
    Wire::from_ordered_points(points).expect("wire is created from more than 2 points")
}

pub fn project_points_to_plane_and_extrude(
    points: impl IntoIterator<Item = Vec3A>,
    plane: &Plane,
    height: f32,
) -> Solid {
    let direction = height * plane.normal();

    wire_from_points(points, plane)
        .to_face()
        .extrude(direction.as_dvec3())
}
