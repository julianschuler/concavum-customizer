use fidget::context::{Context, IntoNode, Node};
use glam::{DAffine3, DVec2, DVec3, Vec3Swizzles};

use crate::model::primitives::{
    vector::{Operations, Vec2, Vec3},
    Result,
};

/// A trait defining Constructive Solid Geometry (CSG) operations.
pub trait Csg {
    /// Performs the union between two shapes.
    fn union<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode;

    /// Performs the difference between two shapes.
    fn difference<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode;

    /// Performs the intersection between two shapes.
    fn intersection<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode;

    /// Extrudes a 2D shape to a given height.
    fn extrude<T: IntoNode>(&mut self, shape: T, height: f64) -> Result<Node>;

    /// Offsets a shape by a given value.
    fn offset<T: IntoNode>(&mut self, shape: T, offset: f64) -> Result<Node>;

    /// Creates a shell with a given thickness.
    fn shell<T: IntoNode>(&mut self, shape: T, thickness: f64) -> Result<Node>;
}

impl Csg for Context {
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

    fn extrude<T: IntoNode>(&mut self, node: T, height: f64) -> Result<Node> {
        let z = self.z();
        let neg_z = self.neg(z)?;
        let diff = self.sub(z, height)?;
        let dist_z = self.max(neg_z, diff)?;

        self.max(node, dist_z)
    }

    fn offset<T: IntoNode>(&mut self, node: T, offset: f64) -> Result<Node> {
        self.sub(node, offset)
    }

    fn shell<T: IntoNode>(&mut self, node: T, thickness: f64) -> Result<Node> {
        let node = node.into_node(self)?;
        let inner = self.offset(node, -thickness)?;

        self.difference(node, inner)
    }
}

/// A trait defining transform operations.
pub trait Transforms {
    /// Translates a shape by a given vector.
    fn translate<T: IntoNode>(&mut self, node: T, translation: DVec3) -> Result<Node>;

    /// Rotates a shape around the x-axis by a given angle.
    fn rotate_x<T: IntoNode>(&mut self, node: T, angle: f64) -> Result<Node>;

    /// Rotates a shape around the y-axis by a given angle.
    fn rotate_y<T: IntoNode>(&mut self, node: T, angle: f64) -> Result<Node>;

    /// Rotates a shape around the z-axis by a given angle.
    fn rotate_z<T: IntoNode>(&mut self, node: T, angle: f64) -> Result<Node>;

    /// Applies an affine linear transform.
    fn affine<T: IntoNode>(&mut self, node: T, affine: DAffine3) -> Result<Node>;

    /// Tapers a shape to the given scale (in x/y) at the given target.
    fn taper<T: IntoNode>(&mut self, node: T, scale: DVec2, target: DVec3) -> Result<Node>;
}

impl Transforms for Context {
    fn translate<T: IntoNode>(&mut self, node: T, translation: DVec3) -> Result<Node> {
        self.affine(node, DAffine3::from_translation(translation))
    }

    fn rotate_x<T: IntoNode>(&mut self, node: T, angle: f64) -> Result<Node> {
        self.affine(node, DAffine3::from_rotation_x(angle))
    }

    fn rotate_y<T: IntoNode>(&mut self, node: T, angle: f64) -> Result<Node> {
        self.affine(node, DAffine3::from_rotation_y(angle))
    }

    fn rotate_z<T: IntoNode>(&mut self, node: T, angle: f64) -> Result<Node> {
        self.affine(node, DAffine3::from_rotation_z(angle))
    }

    fn affine<T: IntoNode>(&mut self, node: T, affine: DAffine3) -> Result<Node> {
        let root = node.into_node(self)?;
        let point = Vec3::point(self);

        // Apply the linear transform
        let matrix = affine.matrix3.inverse().transpose();
        let x_axis = Vec3::from_parameter(self, matrix.x_axis);
        let y_axis = Vec3::from_parameter(self, matrix.y_axis);
        let z_axis = Vec3::from_parameter(self, matrix.z_axis);

        let x = self.vec_dot(point, x_axis)?;
        let y = self.vec_dot(point, y_axis)?;
        let z = self.vec_dot(point, z_axis)?;

        // Apply the translation
        let x = self.sub(x, affine.translation.x)?;
        let y = self.sub(y, affine.translation.y)?;
        let z = self.sub(z, affine.translation.z)?;

        self.remap_xyz(root, [x, y, z])
    }

    fn taper<T: IntoNode>(&mut self, node: T, scale: DVec2, target: DVec3) -> Result<Node> {
        let root = node.into_node(self)?;
        let point = Vec3::point(self);

        // Compute factor for linear interpolation between origin and target
        let z = point.z;
        let alpha = self.div(z, target.z)?;

        // Scale x and y
        let ones = Vec2::from_parameter(self, DVec2::ONE);
        let scale = Vec2::from_parameter(self, scale - DVec2::ONE);
        let scale = self.vec_mul(alpha, scale)?;
        let scale = self.vec_add(scale, ones)?;
        let scaled_x = self.div(point.x, scale.x)?;
        let scaled_y = self.div(point.y, scale.y)?;

        // Shear along x and y
        let target = Vec2::from_parameter(self, target.xy());
        let target = self.vec_mul(alpha, target)?;
        let x = self.sub(scaled_x, target.x)?;
        let y = self.sub(scaled_y, target.y)?;

        self.remap_xyz(root, [x, y, z])
    }
}
