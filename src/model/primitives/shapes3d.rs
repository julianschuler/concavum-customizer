// #![allow(unused)]

use fidget::context::Tree;
use glam::DVec3;

use crate::{
    config::EPSILON,
    model::{
        geometry::Plane,
        primitives::vector::{Vec3, Vector},
    },
};

/// A sphere centered at the origin.
pub struct Sphere {
    radius: f64,
}

impl Sphere {
    /// Creates a new sphere with the given radius.
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }
}

impl From<Sphere> for Tree {
    fn from(sphere: Sphere) -> Self {
        let point = Vec3::point();
        let length = point.length();

        length - sphere.radius
    }
}

/// A half space given by a dividing plane.
pub struct HalfSpace {
    plane: Plane,
}

impl HalfSpace {
    /// Creates a new half given by the dividing plane.
    pub fn new(plane: Plane) -> Self {
        Self { plane }
    }
}

impl From<HalfSpace> for Tree {
    fn from(half_space: HalfSpace) -> Self {
        let point = Vec3::point();
        let normal = Vec3::from_parameter(half_space.plane.normal());
        let plane_point = Vec3::from_parameter(half_space.plane.point());

        let difference = point - plane_point;
        difference.dot(normal)
    }
}

/// A box centered at the origin.
pub struct BoxShape {
    size: DVec3,
}

impl BoxShape {
    /// Creates a new box with the given size.
    pub fn new(size: DVec3) -> Self {
        Self { size }
    }
}

impl From<BoxShape> for Tree {
    fn from(box_shape: BoxShape) -> Self {
        let size = Vec3::from_parameter(box_shape.size / 2.0);
        let abs = Vec3::point().abs();
        let q = abs - size;

        // Use EPSILON instead of 0.0 to get well-behaved gradients
        let max = q.max(EPSILON.into());
        let outer = max.length();

        let max_elem = q.max_elem();
        let inner = max_elem.min(0.0);

        outer + inner
    }
}
