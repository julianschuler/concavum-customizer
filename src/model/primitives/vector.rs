use fidget::{
    context::{IntoNode, Node},
    Context,
};
use glam::{DVec2, DVec3};

use crate::model::primitives::Result;

pub trait Vector {
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
pub struct Vec3 {
    pub x: Node,
    pub y: Node,
    pub z: Node,
}

impl Vec3 {
    pub fn point(context: &mut Context) -> Self {
        let x = context.x();
        let y = context.y();
        let z = context.z();

        Self { x, y, z }
    }

    pub fn from_node<T: IntoNode>(context: &mut Context, node: T) -> Result<Self> {
        let node = node.into_node(context)?;

        Ok(Self {
            x: node,
            y: node,
            z: node,
        })
    }

    pub fn from_parameter(context: &mut Context, parameter: DVec3) -> Self {
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
pub struct Vec2 {
    pub x: Node,
    pub y: Node,
}

impl Vec2 {
    pub fn new(x: Node, y: Node) -> Self {
        Self { x, y }
    }

    pub fn point(context: &mut Context) -> Self {
        let x = context.x();
        let y = context.y();

        Self { x, y }
    }

    pub fn from_node<T: IntoNode>(context: &mut Context, node: T) -> Result<Self> {
        let node = node.into_node(context)?;

        Ok(Self { x: node, y: node })
    }

    pub fn from_parameter(context: &mut Context, parameter: DVec2) -> Self {
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

pub trait Operations {
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

impl Operations for Context {
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
