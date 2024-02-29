use fidget::{
    context::{IntoNode, Node},
    Context, Error,
};
use glam::DVec3;

use crate::model::config::EPSILON;

type Result<T> = std::result::Result<T, Error>;

pub struct Sphere {
    radius: f64,
}

#[allow(unused)]
impl Sphere {
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }
}

impl IntoNode for Sphere {
    fn into_node(self, context: &mut Context) -> Result<Node> {
        let point = Vec::point(context);
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
        let point = Vec::point(context);
        let size = Vec::from_parameter(context, self.size);
        let abs = context.vec_abs(point)?;
        let q = context.vec_sub(abs, size)?;

        // Use EPSILON instead of 0.0 to get well-behaved gradients
        let zero = Vec::from_node(context, EPSILON)?;
        let max = context.vec_max(q, zero)?;
        let outer = context.vec_length(max)?;

        let max_elem = context.vec_max_elem(q)?;
        let inner = context.min(max_elem, 0.0)?;

        context.add(outer, inner)
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
struct Vec {
    x: Node,
    y: Node,
    z: Node,
}

impl Vec {
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

impl Vector for Vec {
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

trait VecOperations {
    /// Calculate the element-wise absolute value
    fn vec_abs<Vec: Vector>(&mut self, a: Vec) -> Result<Vec>;

    /// Calculate the element-wise addition
    fn vec_add<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec>;

    /// Calculate the element-wise subtraction
    fn vec_sub<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec>;

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

    /// Calculate the euclidean length of a Vec
    fn vec_length<Vec: Vector>(&mut self, a: Vec) -> Result<Node>;
}

impl VecOperations for Context {
    fn vec_abs<Vec: Vector>(&mut self, a: Vec) -> Result<Vec> {
        Vec::map_unary(self, Context::abs, a)
    }

    fn vec_add<Vec: Vector>(&mut self, a: Vec, b: Vec) -> Result<Vec> {
        Vec::map_binary(self, Context::add, a, b)
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

    fn vec_length<Vec: Vector>(&mut self, a: Vec) -> Result<Node> {
        let square = self.vec_square(a)?;
        let sum = Vec::fold(self, Context::add, square)?;

        self.sqrt(sum)
    }
}
