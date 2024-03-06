use std::{
    f64::{INFINITY, NEG_INFINITY},
    iter::once,
    vec::Vec,
};

use earcutr::earcut;
use fidget::{
    context::{IntoNode, Node},
    Context,
};
use glam::{dvec2, DVec2, DVec3};

use crate::model::{
    config::EPSILON,
    geometry::counterclockwise_or_colinear,
    primitives::{
        vector::{Operations, Vec2, Vec3, Vector},
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

/// A convex polygon without holes.
/// For non-convex polygons, use [`SimplePolygon`] instead.
pub struct ConvexPolygon {
    vertices: Vec<DVec2>,
}

impl ConvexPolygon {
    /// Creates a new convex polygon from the given vertices.
    /// The vertices have to be in counterclockwise order.
    ///
    /// # Panics
    ///
    /// Panics if there are less than three vertices given.
    pub fn new(vertices: Vec<DVec2>) -> Self {
        assert!(vertices.len() >= 3);

        Self { vertices }
    }

    fn grow_from_triangles(
        start_polygon: [usize; 3],
        triangles: &mut Vec<[usize; 3]>,
        vertices: &[DVec2],
    ) -> Self {
        let mut polygon = start_polygon.to_vec();
        loop {
            let mut triangle_removed = false;

            triangles.retain(|triangle| {
                let &[a, b, c] = triangle;
                let [e1, e2, e3] = [Edge::new(a, b), Edge::new(b, c), Edge::new(c, a)];

                let first = polygon.first().unwrap();
                let last = polygon.last().unwrap();
                polygon
                    .windows(2)
                    .chain(once([*last, *first].as_slice()))
                    .enumerate()
                    .find_map(|(i, window)| {
                        // Try to find a shared edge between polygon and triangle
                        let edge = Edge::new(window[0], window[1]);
                        match edge {
                            edge if edge == e1 => Some((i + 1, c)),
                            edge if edge == e2 => Some((i + 1, a)),
                            edge if edge == e3 => Some((i + 1, b)),
                            _ => None,
                        }
                    })
                    .map_or(true, |(index, point)| {
                        // If resulting polygon is convex, insert point and remove triangle
                        let convex = is_convex_after_insert(&polygon, index, point, vertices);
                        if convex {
                            polygon.insert(index, point);
                            triangle_removed = true;
                        }
                        !convex
                    })
            });

            if !triangle_removed {
                // No triangle was removed, polygon can't grow larger
                break;
            }
        }

        Self {
            vertices: polygon.into_iter().map(|index| vertices[index]).collect(),
        }
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
    /// Creates a new convex polygon from the given vertices.
    /// For non-convex polygons, use [`SimplePolygon`] instead.
    ///
    /// The vertices have to be in a counterclockwise order.
    ///
    /// # Panics
    ///
    /// Panics if there are less than three vertices given.
    pub fn new(vertices: Vec<DVec2>) -> Self {
        assert!(vertices.len() >= 3);

        Self { vertices }
    }

    fn split_into_convex_polygons(&self) -> Vec<ConvexPolygon> {
        let n = self.vertices.len();
        let vertices: Vec<_> = self.vertices.iter().flat_map(DVec2::to_array).collect();
        let triangles =
            earcut(&vertices, &[], 2).expect("simple polygon should always have a triangulation");

        let mut triangles: Vec<[usize; 3]> = triangles
            .chunks_exact(3)
            .map(|slice| slice.try_into().unwrap())
            .collect();
        let mut merged_polygons = Vec::new();

        while let Some(triangle) = triangles.pop() {
            merged_polygons.push(ConvexPolygon::grow_from_triangles(
                triangle,
                &mut triangles,
                &self.vertices,
            ));
        }

        merged_polygons
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

#[derive(Eq, Clone)]
struct Edge {
    start: usize,
    end: usize,
}

impl Edge {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.end && self.end == other.start
    }
}

#[allow(clippy::many_single_char_names)]
fn is_convex_after_insert(
    polygon: &[usize],
    index: usize,
    point: usize,
    vertices: &[DVec2],
) -> bool {
    let n = polygon.len();
    let a = vertices[polygon[(index + n - 2) % n]];
    let b = vertices[polygon[(index + n - 1) % n]];
    let c = vertices[point];
    let d = vertices[polygon[index]];
    let e = vertices[polygon[(index + 1) % n]];

    counterclockwise_or_colinear(a, b, c) && counterclockwise_or_colinear(c, d, e)
}
