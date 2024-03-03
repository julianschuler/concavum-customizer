#![allow(unused)]

use std::{
    f64::{INFINITY, NEG_INFINITY},
    iter::once,
    vec,
};

use fidget::{
    context::{IntoNode, Node},
    Context, Error,
};
use glam::{dvec2, DVec2, DVec3};

use crate::model::config::EPSILON;

type Result<T> = std::result::Result<T, Error>;

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

pub struct ConvexPolygon {
    vertices: vec::Vec<DVec2>,
}

impl ConvexPolygon {
    pub fn new(vertices: vec::Vec<DVec2>) -> Self {
        assert!(vertices.len() >= 3);

        Self { vertices }
    }
}

impl IntoNode for ConvexPolygon {
    fn into_node(self, context: &mut Context) -> Result<Node> {
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
            let vertex_vec = Vec2::from_parameter(context, vertex);
            let diff = context.vec_sub(point, vertex_vec)?;

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
        let abs_distance = context.sqrt(max)?;
        let double_inner = context.mul(2.0, inner)?;

        context.add(abs_distance, double_inner)
    }
}

pub struct Polygon {
    vertices: vec::Vec<DVec2>,
}

impl Polygon {
    pub fn new(vertices: vec::Vec<DVec2>) -> Self {
        assert!(vertices.len() >= 3);

        Self { vertices }
    }
}

impl IntoNode for Polygon {
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

pub trait Csg {
    /// Extrude a 2D shape to a given height
    fn extrusion<T>(&mut self, shape: T, height: f64) -> Result<Node>
    where
        T: IntoNode;

    /// Perform the union between two 3D shapes
    fn union<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode;

    /// Perform the difference between two 3D shapes
    fn difference<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode;

    /// Perform the intersection between two 3D shapes
    fn intersection<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode;
}

impl Csg for Context {
    fn extrusion<T: IntoNode>(&mut self, node: T, height: f64) -> Result<Node> {
        let z = self.z();
        let neg_z = self.neg(z)?;
        let diff = self.sub(z, height)?;
        let dist_z = self.max(neg_z, diff)?;

        self.max(node, dist_z)
    }

    fn union<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode,
    {
        self.min(a, b)
    }

    fn difference<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode,
    {
        let neg_a = self.neg(a)?;
        self.max(neg_a, b)
    }

    fn intersection<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode,
    {
        self.max(a, b)
    }
}

trait Vector {
    /// Apply a unary function element-wise
    fn map_unary<F>(context: &mut Context, f: F, vec: Self) -> Result<Self>
    where
        F: Fn(&mut Context, Node) -> Result<Node>,
        Self: Sized;

    /// Apply a binary function element-wise
    fn map_binary<F>(context: &mut Context, f: F, vec_a: Self, vec_b: Self) -> Result<Self>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>,
        Self: Sized;

    /// Fold all Vec elements using a binary function
    fn fold<F>(context: &mut Context, f: F, vec: Self) -> Result<Node>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>,
        Self: Sized;
}

#[derive(Copy, Clone)]
struct Vec3 {
    x: Node,
    y: Node,
    z: Node,
}

impl Vec3 {
    fn point(context: &mut Context) -> Self {
        let x = context.x();
        let y = context.y();
        let z = context.z();

        Self { x, y, z }
    }

    fn from_node<T: IntoNode>(context: &mut Context, node: T) -> Result<Self> {
        let node = node.into_node(context)?;

        Ok(Self {
            x: node,
            y: node,
            z: node,
        })
    }

    fn from_parameter(context: &mut Context, parameter: DVec3) -> Self {
        let x = context.constant(parameter.x);
        let y = context.constant(parameter.y);
        let z = context.constant(parameter.z);

        Self { x, y, z }
    }
}

impl Vector for Vec3 {
    fn map_unary<F>(context: &mut Context, f: F, vec: Self) -> Result<Self>
    where
        F: Fn(&mut Context, Node) -> Result<Node>,
        Self: Sized,
    {
        let x = f(context, vec.x)?;
        let y = f(context, vec.y)?;
        let z = f(context, vec.z)?;

        Ok(Self { x, y, z })
    }

    fn map_binary<F>(context: &mut Context, f: F, vec_a: Self, vec_b: Self) -> Result<Self>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>,
        Self: Sized,
    {
        let x = f(context, vec_a.x, vec_b.x)?;
        let y = f(context, vec_a.y, vec_b.y)?;
        let z = f(context, vec_a.z, vec_b.z)?;

        Ok(Self { x, y, z })
    }

    fn fold<F>(context: &mut Context, f: F, vec: Self) -> Result<Node>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>,
        Self: Sized,
    {
        let result = f(context, vec.x, vec.y)?;
        f(context, result, vec.z)
    }
}

#[derive(Copy, Clone)]
struct Vec2 {
    x: Node,
    y: Node,
}

impl Vec2 {
    fn new(x: Node, y: Node) -> Self {
        Self { x, y }
    }

    fn point(context: &mut Context) -> Self {
        let x = context.x();
        let y = context.y();

        Self { x, y }
    }

    fn from_node<T: IntoNode>(context: &mut Context, node: T) -> Result<Self> {
        let node = node.into_node(context)?;

        Ok(Self { x: node, y: node })
    }

    fn from_parameter(context: &mut Context, parameter: DVec2) -> Self {
        let x = context.constant(parameter.x);
        let y = context.constant(parameter.y);

        Self { x, y }
    }
}

impl Vector for Vec2 {
    fn map_unary<F>(context: &mut Context, f: F, vec: Self) -> Result<Self>
    where
        F: Fn(&mut Context, Node) -> Result<Node>,
        Self: Sized,
    {
        let x = f(context, vec.x)?;
        let y = f(context, vec.y)?;

        Ok(Self { x, y })
    }

    fn map_binary<F>(context: &mut Context, f: F, vec_a: Self, vec_b: Self) -> Result<Self>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>,
        Self: Sized,
    {
        let x = f(context, vec_a.x, vec_b.x)?;
        let y = f(context, vec_a.y, vec_b.y)?;

        Ok(Self { x, y })
    }

    fn fold<F>(context: &mut Context, f: F, vec: Self) -> Result<Node>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>,
        Self: Sized,
    {
        f(context, vec.x, vec.y)
    }
}

trait VectorOperations {
    /// Calculate the element-wise absolute value
    fn vec_abs<Vec: Vector>(&mut self, a: Vec) -> Result<Vec>;

    /// Calculate the element-wise addition
    fn vec_add<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec>;

    /// Calculate the element-wise subtraction
    fn vec_sub<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec>;

    /// Calculate the scalar multiplication
    fn vec_mul<Vec: Vector>(&mut self, scalar: Node, a: Vec) -> Result<Vec>;

    /// Square each element of a Vec
    fn vec_square<Vec: Vector>(&mut self, a: Vec) -> Result<Vec>;

    /// Calculate the element-wise mininum
    fn vec_min<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec>;

    /// Calculate the element-wise maximum
    fn vec_max<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec>;

    /// Calculate the minimum value of all elements
    fn vec_min_elem<Vec: Vector>(&mut self, a: Vec) -> Result<Node>;

    /// Calculate the maximum value of all elements
    fn vec_max_elem<Vec: Vector>(&mut self, a: Vec) -> Result<Node>;

    /// Calculate the dot product of two Vecs
    fn vec_dot<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Node>;

    /// Calculate the euclidean norm of a Vec
    fn vec_length<Vec: Vector + Copy>(&mut self, a: Vec) -> Result<Node>;
}

impl VectorOperations for Context {
    fn vec_abs<Vec: Vector>(&mut self, a: Vec) -> Result<Vec> {
        Vec::map_unary(self, Context::abs, a)
    }

    fn vec_add<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec> {
        Vec::map_binary(self, Context::add, a, b)
    }

    fn vec_mul<Vec: Vector>(&mut self, scalar: Node, a: Vec) -> Result<Vec> {
        Vec::map_unary(self, |context, x| context.mul(scalar, x), a)
    }

    fn vec_sub<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec> {
        Vec::map_binary(self, Context::sub, a, b)
    }

    fn vec_square<Vec: Vector>(&mut self, a: Vec) -> Result<Vec> {
        Vec::map_unary(self, Context::square, a)
    }

    fn vec_min<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec> {
        Vec::map_binary(self, Context::min, a, b)
    }

    fn vec_max<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec> {
        Vec::map_binary(self, Context::max, a, b)
    }

    fn vec_min_elem<Vec: Vector>(&mut self, a: Vec) -> Result<Node> {
        Vec::fold(self, Context::min, a)
    }

    fn vec_max_elem<Vec: Vector>(&mut self, a: Vec) -> Result<Node> {
        Vec::fold(self, Context::max, a)
    }

    fn vec_dot<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Node> {
        let product = Vec::map_binary(self, Context::mul, a, b)?;
        Vec::fold(self, Context::add, product)
    }

    fn vec_length<Vec: Vector + Copy>(&mut self, a: Vec) -> Result<Node> {
        let squared_length = self.vec_dot(a, a)?;
        self.sqrt(squared_length)
    }
}
