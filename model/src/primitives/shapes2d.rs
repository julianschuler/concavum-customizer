use std::{iter::once, vec::Vec};

use earcutr::earcut;
use fidget::context::Tree;
use glam::DVec2;

use crate::{
    geometry::{counterclockwise_or_colinear, rotate_90_degrees},
    primitives::{
        vector::{Vec2, Vector},
        EPSILON,
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
        let q = Vec2::point().abs() - (rectangle.size / 2.0).into();

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

    /// Grows a convex polygon from a triangulation of a start triangle by iteratively
    /// merging it with adjacent triangles that result in covex polygons after merging.
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

                let first = *polygon.first().expect("there are at least three vertices");
                let last = *polygon.last().expect("there are at least three vertices");
                polygon
                    .windows(2)
                    .chain(once([last, first].as_slice()))
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
                        // If the resulting polygon is convex, insert point and remove triangle
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

    /// Returns the distances of the polygon.
    fn distances(self) -> Distances {
        let point = Vec2::point();
        let first = *self
            .vertices
            .first()
            .expect("there are at least three vertices");
        let last = *self
            .vertices
            .last()
            .expect("there are at least three vertices");

        self.vertices
            .windows(2)
            .chain(once([last, first].as_slice()))
            .map(|window| {
                let previous_vertex = window[0];
                let vertex = window[1];

                let edge = previous_vertex - vertex;
                let edge_length = edge.length();
                let edge = edge.normalize_or_zero();
                let normal: Vec2 = rotate_90_degrees(edge).into();
                let edge: Vec2 = edge.into();
                let diff = point.clone() - vertex.into();

                // Calculate shortest possible vector from point to edge
                let edge_projection = diff.dot(edge.clone());
                let clamped_factor = edge_projection.max(0.0).min(edge_length);
                let shortest_diff = diff.clone() - clamped_factor * edge;

                let squared = shortest_diff.squared_length();
                let inner = normal.dot(diff).min(0.0);

                Distances { squared, inner }
            })
            .reduce(Distances::combine_convex)
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

    /// Splits the simple polygon into a list of convex polygons.
    fn split_into_convex_polygons(&self) -> Vec<ConvexPolygon> {
        let vertices: Vec<_> = self.vertices.iter().flat_map(DVec2::to_array).collect();
        let triangles =
            earcut(&vertices, &[], 2).expect("simple polygon should always have a triangulation");

        let mut triangles: Vec<[usize; 3]> = triangles
            .chunks_exact(3)
            .map(|slice| slice.try_into().expect("the slice has three elements"))
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
        let first = *polygon
            .vertices
            .first()
            .expect("there are at least three vertices");
        let last = *polygon
            .vertices
            .last()
            .expect("there are at least three vertices");

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
                let edge: Vec2 = edge.into();
                let diff = point.clone() - vertex.into();

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

/// A corner at the origin with two outgoing edges.
pub struct Corner {
    edge1: DVec2,
    edge2: DVec2,
}

impl Corner {
    /// Creates a new corner at the origin given by two outgoing edges.
    /// The left side of the first edge is considered the exterior.
    ///
    /// # Panics
    ///
    /// Panics if the edges have length zero and `glam_assert` is enabled.
    pub fn new(edge1: DVec2, edge2: DVec2) -> Self {
        Self {
            edge1: edge1.normalize(),
            edge2: edge2.normalize(),
        }
    }
}

impl From<Corner> for Tree {
    fn from(corner: Corner) -> Self {
        let point = Vec2::point();
        let edge1: Vec2 = corner.edge1.into();
        let edge2: Vec2 = corner.edge2.into();
        let normal1 = rotate_90_degrees(-corner.edge1).into();
        let normal2 = rotate_90_degrees(corner.edge2).into();

        let Distances { squared, inner } = [(edge1, normal1), (edge2, normal2)]
            .into_iter()
            .map(|(edge, normal)| {
                let edge_projection = point.dot(edge.clone());
                let shortest_diff = point.clone() - edge_projection.max(0.0) * edge;
                let squared = shortest_diff.squared_length();
                let inner = point.dot(normal).min(0.0);

                Distances { squared, inner }
            })
            .reduce(Distances::combine_convex)
            .expect("there are 2 edge normal pairs");

        squared.max(EPSILON).sqrt() + 2.0 * inner
    }
}

/// A helper struct containing a squared and signed inner distance.
struct Distances {
    squared: Tree,
    inner: Tree,
}

impl Distances {
    /// Performs a convex combination of two distances.
    fn combine_convex(self, other: Self) -> Self {
        Distances {
            squared: self.squared.min(other.squared),
            inner: self.inner.max(other.inner),
        }
    }
}

/// An edge given by two vertex indices.
#[derive(PartialEq, Eq, Clone)]
struct Edge {
    start: usize,
    end: usize,
}

impl Edge {
    /// Creates a new edge from the given start and end indices.
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Returns true if the given polygon is convex after inserting a new point.
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
