use fidget::context::{Context, IntoNode, Node};
use glam::{DAffine3, DVec2, DVec3};

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

    /// Extrudes a 2D shape between two `z_min` and `z_max`.
    fn extrude<T: IntoNode>(&mut self, shape: T, z_min: f64, z_max: f64) -> Result<Node>;

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
        let neg_b = self.neg(b)?;
        self.max(a, neg_b)
    }

    fn intersection<A, B>(&mut self, a: A, b: B) -> Result<Node>
    where
        A: IntoNode,
        B: IntoNode,
    {
        self.max(a, b)
    }

    fn extrude<T: IntoNode>(&mut self, node: T, z_min: f64, z_max: f64) -> Result<Node> {
        let z = self.z();
        let dist_z_min = self.sub(z_min, z)?;
        let dist_z_max = self.sub(z, z_max)?;
        let dist_z = self.max(dist_z_min, dist_z_max)?;

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

    /// Tapers a shape to the given scale (in x/y) at the given height.
    fn taper<T: IntoNode>(&mut self, node: T, scale: DVec2, height: f64) -> Result<Node>;

    /// Shears a shape to the given offset (in x/y) at the given height.
    fn shear<T: IntoNode>(&mut self, node: T, offset: DVec2, height: f64) -> Result<Node>;
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

    fn taper<T: IntoNode>(&mut self, node: T, scale: DVec2, height: f64) -> Result<Node> {
        let root = node.into_node(self)?;
        let point = Vec3::point(self);
        let alpha = self.div(point.z, height)?;

        // Scale x and y
        let ones = Vec2::from_parameter(self, DVec2::ONE);
        let scale = Vec2::from_parameter(self, scale - DVec2::ONE);
        let scale = self.vec_mul(alpha, scale)?;
        let scale = self.vec_add(scale, ones)?;
        let x = self.div(point.x, scale.x)?;
        let y = self.div(point.y, scale.y)?;

        self.remap_xyz(root, [x, y, point.z])
    }

    fn shear<T: IntoNode>(&mut self, node: T, offset: DVec2, height: f64) -> Result<Node> {
        let root = node.into_node(self)?;
        let point = Vec3::point(self);
        let alpha = self.div(point.z, height)?;

        // Shear along x and y
        let offset = Vec2::from_parameter(self, offset);
        let offset = self.vec_mul(alpha, offset)?;
        let x = self.sub(point.x, offset.x)?;
        let y = self.sub(point.y, offset.y)?;

        self.remap_xyz(root, [x, y, point.z])
    }
}
