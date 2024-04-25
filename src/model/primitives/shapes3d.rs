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
        Vec3::point().length() - sphere.radius
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
        let normal = Vec3::from_parameter(half_space.plane.normal());
        let difference = Vec3::point() - Vec3::from_parameter(half_space.plane.point());

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
        let q = Vec3::point().abs() - size;

        // Use EPSILON instead of 0.0 to get well-behaved gradients
        let outer = q.max(EPSILON.into()).length();
        let inner = q.max_elem().min(0.0);

        outer + inner
    }
}
