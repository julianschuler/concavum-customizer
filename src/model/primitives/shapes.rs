use std::{
    f64::{INFINITY, NEG_INFINITY},
    iter::once,
    result, vec,
};

use fidget::{
    context::{IntoNode, Node},
    Context,
};
use glam::{dvec2, DVec2, DVec3};

use crate::model::{
    config::EPSILON,
    primitives::{
        vector::{Operations, Vec2, Vec3, Vector},
        Result,
    },
};

pub struct Sphere {
    radius: f64,
}

impl Sphere {
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

pub struct BoxShape {
    size: DVec3,
}

impl BoxShape {
    pub fn new(size: DVec3) -> Self {
        Self { size }
    }
}

impl IntoNode for BoxShape {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let point = Vec3::point(context);
        let size = Vec3::from_parameter(context, self.size);
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

struct Distances {
    absolute: Node,
    inner: Node,
}

pub struct ConvexPolygon {
    vertices: Vec<DVec2>,
}

impl ConvexPolygon {
    pub fn new(vertices: Vec<DVec2>) -> Self {
        assert!(vertices.len() >= 3);

        Self { vertices }
    }

    fn distances(self, context: &mut Context) -> Result<Distances> {
        let point = Vec2::point(context);
        let length = context.vec_length(point)?;

        let first = *self.vertices.first().unwrap();
        let last = *self.vertices.last().unwrap();

        let mut squared_abs = context.constant(INFINITY);
        let mut inner = context.constant(NEG_INFINITY);

        for window in self
            .vertices
            .windows(2)
            .chain(once([last, first].as_slice()))
        {
            let previous_vertex = window[0];
            let vertex = window[1];

            let edge = previous_vertex - vertex;
            let edge_length = edge.length();
            let edge = edge.normalize_or_zero();
            let normal = Vec2::from_parameter(context, dvec2(-edge.y, edge.x));
            let edge = Vec2::from_parameter(context, edge);
            let vertex = Vec2::from_parameter(context, vertex);
            let diff = context.vec_sub(point, vertex)?;

            // Calculate shortest possible vector from point to edge
            let edge_projection = context.vec_dot(diff, edge)?;
            let max = context.max(edge_projection, 0.0)?;
            let clamped_factor = context.min(max, edge_length)?;
            let scaled_edge = context.vec_mul(clamped_factor, edge)?;
            let shortest_diff = context.vec_sub(diff, scaled_edge)?;

            let shortest_distance = context.vec_dot(shortest_diff, shortest_diff)?;
            squared_abs = context.min(squared_abs, shortest_distance)?;

            // Calculate inner distance
            let dot = context.vec_dot(normal, diff)?;
            let min = context.min(dot, 0.0)?;
            inner = context.max(inner, min)?;
        }

        // Clamp to EPSILON to get well-behaved gradients
        let max = context.max(squared_abs, EPSILON)?;
        let absolute_distance = context.sqrt(max)?;

        Ok(Distances {
            absolute: absolute_distance,
            inner,
        })
    }
}

impl IntoNode for ConvexPolygon {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let Distances { absolute, inner } = self.distances(context)?;
        let double_inner = context.mul(2.0, inner)?;

        context.add(absolute, double_inner)
    }
}

pub struct SimplePolygon {
    vertices: Vec<DVec2>,
}

impl SimplePolygon {
    pub fn new(vertices: Vec<DVec2>) -> Self {
        assert!(vertices.len() >= 3);

        Self { vertices }
    }
}

impl IntoNode for SimplePolygon {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let point = Vec2::point(context);
        let length = context.vec_length(point)?;

        let vertices: Vec<_> = self
            .vertices
            .into_iter()
            .map(|vertex| Vec2::from_parameter(context, vertex))
            .collect();

        let first = vertices.first().unwrap();
        let last = vertices.last().unwrap();
        let diff = context.vec_sub(point, *first)?;

        let mut squared_distance = context.vec_dot(diff, diff)?;
        let mut sign = context.constant(1.0);

        for window in vertices.windows(2).chain(once([*last, *first].as_slice())) {
            let previous_vertex = window[0];
            let vertex = window[1];

            let edge = context.vec_sub(previous_vertex, vertex)?;
            let diff = context.vec_sub(point, vertex)?;

            // Calculate shortest possible vector from point to edge
            let edge_projection = context.vec_dot(diff, edge)?;
            let edge_length_squared = context.vec_dot(edge, edge)?;
            let closest_point_factor = context.div(edge_projection, edge_length_squared)?;

            let max = context.max(closest_point_factor, 0.0)?;
            let clamped_factor = context.min(max, 1.0)?;
            let scaled_edge = context.vec_mul(clamped_factor, edge)?;
            let shortest_diff = context.vec_sub(diff, scaled_edge)?;

            let shortest_distance = context.vec_dot(shortest_diff, shortest_diff)?;
            squared_distance = context.min(squared_distance, shortest_distance)?;

            // Calculate winding number (determine whether point is inside or outside polygon)
            let point_above_vertex = context.less_than(vertex.y, point.y)?;
            let point_below_previous_vertex = context.less_than(point.y, previous_vertex.y)?;
            let m1 = context.mul(edge.y, diff.x)?;
            let m2 = context.mul(edge.x, diff.y)?;
            let m1_less_than_m2 = context.less_than(m1, m2)?;

            // Multiply sign by -1.0 if all or none of the conditions are true
            let sum = context.add(point_above_vertex, point_below_previous_vertex)?;
            let sum = context.add(sum, m1_less_than_m2)?;
            let shifted = context.sub(sum, 1.5)?;
            let abs = context.abs(shifted)?;
            let indicator = context.sub(1.5, abs)?;
            let scaled = context.mul(2.0, indicator)?;
            let indicator = context.sub(scaled, 1.0)?;
            sign = context.mul(sign, indicator)?;
        }

        let distance = context.sqrt(squared_distance)?;
        context.mul(sign, distance)
    }
}
