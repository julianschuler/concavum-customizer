#![allow(unused)]

use fidget::{
    context::{IntoNode, Node},
    Context,
};
use glam::DVec3;

use crate::model::{
    config::EPSILON,
    geometry::Plane,
    primitives::{
        vector::{Operations, Vec3, Vector},
        Result,
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

impl IntoNode for Sphere {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let point = Vec3::point(context);
        let length = context.vec_length(point)?;

        context.sub(length, self.radius)
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

impl IntoNode for HalfSpace {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let point = Vec3::point(context);
        let normal = Vec3::from_parameter(context, self.plane.normal());
        let plane_point = Vec3::from_parameter(context, self.plane.point());

        let difference = context.vec_sub(point, plane_point)?;
        context.vec_dot(difference, normal)
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

impl IntoNode for BoxShape {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let point = Vec3::point(context);
        let size = Vec3::from_parameter(context, self.size / 2.0);
        let abs = context.vec_abs(point)?;
        let q = context.vec_sub(abs, size)?;

        // Use EPSILON instead of 0.0 to get well-behaved gradients
        let zero = Vec3::from_node(context, EPSILON)?;
        let max = context.vec_max(q, zero)?;
        let outer = context.vec_length(max)?;

        let max_elem = context.vec_max_elem(q)?;
        let inner = context.min(max_elem, 0.0)?;

        context.add(outer, inner)
    }
}
