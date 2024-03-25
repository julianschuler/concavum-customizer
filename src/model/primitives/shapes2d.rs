#![allow(unused)]

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
use glam::{dvec2, DVec2};

use crate::model::{
    config::EPSILON,
    geometry::counterclockwise_or_colinear,
    primitives::{
        vector::{Operations, Vec2, Vector},
        Result,
    },
};

/// A circle centered at the origin.
pub struct Circle {
    radius: f64,
}

impl Circle {
    /// Creates a new circle with the given radius.
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }
}

impl IntoNode for Circle {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let point = Vec2::point(context);
        let length = context.vec_length(point)?;

        context.sub(length, self.radius)
    }
}

/// A rectangle centered at the origin.
pub struct Rectangle {
    size: DVec2,
}

impl Rectangle {
    /// Creates a new rectangle with the given size.
    pub fn new(size: DVec2) -> Self {
        Self { size }
    }
}

impl IntoNode for Rectangle {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let point = Vec2::point(context);
        let size = Vec2::from_parameter(context, self.size / 2.0);
        let abs = context.vec_abs(point)?;
        let q = context.vec_sub(abs, size)?;

        // Use EPSILON instead of 0.0 to get well-behaved gradients
        let zero = Vec2::from_node(context, EPSILON)?;
        let max = context.vec_max(q, zero)?;
        let outer = context.vec_length(max)?;

        let max_elem = context.vec_max_elem(q)?;
        let inner = context.min(max_elem, 0.0)?;

        context.add(outer, inner)
    }
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
                        let edge = Edge::new(window[1], window[0]);
                        match edge {
                            edge if edge == e1 => Some((i, c)),
                            edge if edge == e2 => Some((i, a)),
                            edge if edge == e3 => Some((i, b)),
                            _ => None,
                        }
                    })
                    .map_or(true, |(index, point)| {
                        // If resulting polygon is convex, insert point and remove triangle
                        let convex = is_convex_after_insert(&polygon, index, point, vertices);
                        if convex {
                            polygon.insert(index + 1, point);
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
        let first = *self.vertices.first().unwrap();
        let last = *self.vertices.last().unwrap();

        let mut squared = context.constant(INFINITY);
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

            let shortest_squared = context.vec_dot(shortest_diff, shortest_diff)?;
            squared = context.min(squared, shortest_squared)?;

            // Calculate inner distance
            let dot = context.vec_dot(normal, diff)?;
            let min = context.min(dot, 0.0)?;
            inner = context.max(inner, min)?;
        }

        Ok(Distances { squared, inner })
    }
}

impl IntoNode for ConvexPolygon {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let Distances { squared, inner } = self.distances(context)?;

        // Clamp to EPSILON to get well-behaved gradients
        let clamped_squared = context.max(squared, EPSILON)?;
        let absolute_distance = context.sqrt(clamped_squared)?;
        let double_inner = context.mul(2.0, inner)?;

        context.add(absolute_distance, double_inner)
    }
}

/// A simple polygon without self-intersections or holes.
/// If the polygon is convex, consider using [`ConvexPolygon`] instead.
pub struct SimplePolygon {
    vertices: Vec<DVec2>,
}

impl SimplePolygon {
    /// Creates a new simple polygon from the given vertices.
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
        let first = *self.vertices.first().unwrap();
        let last = *self.vertices.last().unwrap();

        let mut squared = context.constant(INFINITY);

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
            let edge = Vec2::from_parameter(context, edge);
            let vertex = Vec2::from_parameter(context, vertex);
            let diff = context.vec_sub(point, vertex)?;

            // Calculate shortest possible vector from point to edge
            let edge_projection = context.vec_dot(diff, edge)?;
            let max = context.max(edge_projection, 0.0)?;
            let clamped_factor = context.min(max, edge_length)?;
            let scaled_edge = context.vec_mul(clamped_factor, edge)?;
            let shortest_diff = context.vec_sub(diff, scaled_edge)?;

            let shortest_squared = context.vec_dot(shortest_diff, shortest_diff)?;
            squared = context.min(squared, shortest_squared)?;
        }

        // Calculate outer distance from outer distances of convex partition
        let mut squared_outer_distance = context.constant(INFINITY);
        for polygon in self.split_into_convex_polygons() {
            let Distances { squared, inner } = polygon.distances(context)?;
            let squared_inner = context.square(inner)?;
            let squared_outer = context.sub(squared, squared_inner)?;

            squared_outer_distance = context.min(squared_outer_distance, squared_outer)?;
        }

        // Clamp to EPSILON to get well-behaved gradients
        let clamped_squared_outer = context.max(squared_outer_distance, EPSILON)?;
        let outer_distance = context.sqrt(clamped_squared_outer)?;
        let double_outer = context.mul(2.0, outer_distance)?;

        // Clamp to EPSILON to get well-behaved gradients
        let clamped_squared = context.max(squared, EPSILON)?;
        let absolute_distance = context.sqrt(clamped_squared)?;

        context.sub(double_outer, absolute_distance)
    }
}

struct Distances {
    squared: Node,
    inner: Node,
}

#[derive(PartialEq, Eq, Clone)]
struct Edge {
    start: usize,
    end: usize,
}

impl Edge {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
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
    let a = vertices[polygon[(index + n - 1) % n]];
    let b = vertices[polygon[index]];
    let c = vertices[point];
    let d = vertices[polygon[(index + 1) % n]];
    let e = vertices[polygon[(index + 2) % n]];

    counterclockwise_or_colinear(a, b, c) && counterclockwise_or_colinear(c, d, e)
}
