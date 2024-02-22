use fidget::{
    context::{IntoNode, Node},
    Context, Error,
};
use glam::DVec3;

type Result<T> = std::result::Result<T, Error>;

pub struct Sphere {
    radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
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

trait VecOperations {
    /// Apply a unary function element-wise
    fn map_unary<F>(&mut self, f: F, a: Vec) -> Result<Vec>
    where
        F: Fn(&mut Context, Node) -> Result<Node>;

    /// Apply a unary function element-wise
    fn map_binary<F>(&mut self, f: F, a: Vec, b: Vec) -> Result<Vec>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>;

    /// Fold all Vec elements using a binary function
    fn fold<F>(&mut self, f: F, a: Vec) -> Result<Node>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>;

    /// Calculate the element-wise absolute value
    fn vec_abs(&mut self, a: Vec) -> Result<Vec>;

    /// Calculate the element-wise addition
    fn vec_add(&mut self, a: Vec, b: Vec) -> Result<Vec>;

    /// Calculate the element-wise subtraction
    fn vec_sub(&mut self, a: Vec, b: Vec) -> Result<Vec>;

    /// Square each element of a Vec
    fn vec_square(&mut self, a: Vec) -> Result<Vec>;

    /// Calculate the element-wise mininum
    fn vec_min(&mut self, a: Vec, b: Vec) -> Result<Vec>;

    /// Calculate the element-wise maximum
    fn vec_max(&mut self, a: Vec, b: Vec) -> Result<Vec>;

    /// Calculate the minimum value of all elements
    fn vec_min_elem(&mut self, a: Vec) -> Result<Node>;

    /// Calculate the maximum value of all elements
    fn vec_max_elem(&mut self, a: Vec) -> Result<Node>;

    /// Calculate the euclidean length of a Vec
    fn vec_length(&mut self, a: Vec) -> Result<Node>;
}

impl VecOperations for Context {
    fn map_unary<F>(&mut self, f: F, a: Vec) -> Result<Vec>
    where
        F: Fn(&mut Context, Node) -> Result<Node>,
    {
        let x = f(self, a.x)?;
        let y = f(self, a.y)?;
        let z = f(self, a.z)?;

        Ok(Vec { x, y, z })
    }

    fn map_binary<F>(&mut self, f: F, a: Vec, b: Vec) -> Result<Vec>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>,
    {
        let x = f(self, a.x, b.x)?;
        let y = f(self, a.y, b.y)?;
        let z = f(self, a.z, b.z)?;

        Ok(Vec { x, y, z })
    }

    fn fold<F>(&mut self, f: F, a: Vec) -> Result<Node>
    where
        F: Fn(&mut Context, Node, Node) -> Result<Node>,
    {
        let result = f(self, a.x, a.y)?;
        f(self, result, a.z)
    }

    fn vec_abs(&mut self, a: Vec) -> Result<Vec> {
        self.map_unary(Context::abs, a)
    }

    fn vec_add(&mut self, a: Vec, b: Vec) -> Result<Vec> {
        self.map_binary(Context::add, a, b)
    }

    fn vec_sub(&mut self, a: Vec, b: Vec) -> Result<Vec> {
        self.map_binary(Context::sub, a, b)
    }

    fn vec_square(&mut self, a: Vec) -> Result<Vec> {
        self.map_unary(Context::square, a)
    }

    fn vec_min(&mut self, a: Vec, b: Vec) -> Result<Vec> {
        self.map_binary(Context::min, a, b)
    }

    fn vec_max(&mut self, a: Vec, b: Vec) -> Result<Vec> {
        self.map_binary(Context::max, a, b)
    }

    fn vec_min_elem(&mut self, a: Vec) -> Result<Node> {
        self.fold(Context::min, a)
    }

    fn vec_max_elem(&mut self, a: Vec) -> Result<Node> {
        self.fold(Context::max, a)
    }

    fn vec_length(&mut self, a: Vec) -> Result<Node> {
        let square = self.vec_square(a)?;
        let sum = self.fold(Context::add, square)?;

        self.sqrt(sum)
    }
}