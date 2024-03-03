use fidget::context::{Context, IntoNode, Node};

use crate::model::primitives::Result;

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
