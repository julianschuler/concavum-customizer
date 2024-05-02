use std::{iter::once, vec::Vec};

use earcutr::earcut;
use fidget::context::Tree;
use glam::DVec2;

use crate::{
    config::EPSILON,
    model::{
        geometry::{counterclockwise_or_colinear, rotate_90_degrees},
        primitives::vector::{Vec2, Vector},
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

impl From<Circle> for Tree {
    fn from(circle: Circle) -> Self {
        let point = Vec2::point();
        let length = point.length();

        length - circle.radius
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

impl From<Rectangle> for Tree {
    fn from(rectangle: Rectangle) -> Self {
        let size = Vec2::from_parameter(rectangle.size / 2.0);
        let q = Vec2::point().abs() - size;

        // Use EPSILON instead of 0.0 to get well-behaved gradients
        let outer = q.max(EPSILON.into()).length();
        let inner = q.max_elem().min(0.0);

        outer + inner
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

    fn distances(self) -> Distances {
        let point = Vec2::point();
        let first = *self.vertices.first().unwrap();
        let last = *self.vertices.last().unwrap();

        self.vertices
            .windows(2)
            .chain(once([last, first].as_slice()))
            .map(|window| {
                let previous_vertex = window[0];
                let vertex = window[1];

                let edge = previous_vertex - vertex;
                let edge_length = edge.length();
                let edge = edge.normalize_or_zero();
                let normal = Vec2::from_parameter(rotate_90_degrees(edge));
                let edge = Vec2::from_parameter(edge);
                let diff = point.clone() - Vec2::from_parameter(vertex);

                // Calculate shortest possible vector from point to edge
                let edge_projection = diff.dot(edge.clone());
                let clamped_factor = edge_projection.max(0.0).min(edge_length);
                let shortest_diff = diff.clone() - clamped_factor * edge;

                let squared = shortest_diff.squared_length();
                let inner = normal.dot(diff).min(0.0);

                Distances { squared, inner }
            })
            .reduce(Distances::combine)
            .expect("there is always a vertex")
    }
}

impl From<ConvexPolygon> for Tree {
    fn from(polygon: ConvexPolygon) -> Self {
        let Distances { squared, inner } = polygon.distances();

        // Clamp to EPSILON to get well-behaved gradients
        squared.max(EPSILON).sqrt() + 2.0 * inner
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

impl From<SimplePolygon> for Tree {
    fn from(polygon: SimplePolygon) -> Self {
        let point = Vec2::point();
        let first = *polygon.vertices.first().unwrap();
        let last = *polygon.vertices.last().unwrap();

        let squared = polygon
            .vertices
            .windows(2)
            .chain(once([last, first].as_slice()))
            .map(|window| {
                let previous_vertex = window[0];
                let vertex = window[1];

                let edge = previous_vertex - vertex;
                let edge_length = edge.length();
                let edge = edge.normalize_or_zero();
                let edge = Vec2::from_parameter(edge);
                let diff = point.clone() - Vec2::from_parameter(vertex);

                // Calculate shortest possible vector from point to edge
                let edge_projection = diff.dot(edge.clone());
                let clamped_factor = edge_projection.max(0.0).min(edge_length);
                let shortest_diff = diff - clamped_factor * edge;

                shortest_diff.squared_length()
            })
            .reduce(|a, b| a.min(b))
            .expect("there is always a vertex");

        // Calculate outer distance from outer distances of convex partition
        let squared_outer_distance = polygon
            .split_into_convex_polygons()
            .into_iter()
            .map(|polygon| {
                let Distances { squared, inner } = polygon.distances();
                squared - inner.square()
            })
            .reduce(|a, b| a.min(b))
            .expect("there is always a polygon");

        // Clamp to EPSILON to get well-behaved gradients
        let outer_distance = squared_outer_distance.max(EPSILON).sqrt();
        let absolute_distance = squared.max(EPSILON).sqrt();

        2.0 * outer_distance - absolute_distance
    }
}

struct Distances {
    squared: Tree,
    inner: Tree,
}

impl Distances {
    fn combine(self, other: Self) -> Self {
        Distances {
            squared: self.squared.min(other.squared),
            inner: self.inner.max(other.inner),
        }
    }
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
