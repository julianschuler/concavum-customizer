use fidget::context::Tree;
use glam::{DAffine3, DVec2, DVec3};

use crate::model::primitives::vector::{Vec2, Vec3};

use super::vector::Vector;

/// A trait defining Constructive Solid Geometry (CSG) operations.
pub trait Csg {
    /// Performs the union between two shapes.
    fn union<T: Into<Tree>>(&self, other: T) -> Self;

    /// Performs the difference between two shapes.
    fn difference<T: Into<Tree>>(&self, other: T) -> Self;

    /// Performs the intersection between two shapes.
    fn intersection<T: Into<Tree>>(&self, other: T) -> Self;

    /// Extrudes a 2D shape between two `z_min` and `z_max`.
    fn extrude(&self, z_min: f64, z_max: f64) -> Self;

    /// Offsets a shape by a given value.
    fn offset(&self, offset: f64) -> Self;

    /// Creates a shell with a given thickness.
    fn shell(&self, thickness: f64) -> Self;
}

impl Csg for Tree {
    fn union<T: Into<Tree>>(&self, other: T) -> Self {
        self.min(other)
    }

    fn difference<T: Into<Tree>>(&self, other: T) -> Self {
        self.max(other.into().neg())
    }

    fn intersection<T: Into<Tree>>(&self, other: T) -> Self {
        self.max(other)
    }

    fn extrude(&self, z_min: f64, z_max: f64) -> Self {
        self.max((z_min - Tree::z()).max(Tree::z() - z_max))
    }

    fn offset(&self, offset: f64) -> Self {
        self.clone() - offset
    }

    fn shell(&self, thickness: f64) -> Self {
        self.difference(self.offset(-thickness))
    }
}

/// A trait defining transform operations.
pub trait Transforms {
    /// Translates a shape by a given vector.
    fn translate(&self, translation: DVec3) -> Tree;

    /// Rotates a shape around the x-axis by a given angle.
    fn rotate_x(&self, angle: f64) -> Tree;

    /// Rotates a shape around the y-axis by a given angle.
    fn rotate_y(&self, angle: f64) -> Tree;

    /// Rotates a shape around the z-axis by a given angle.
    fn rotate_z(&self, angle: f64) -> Tree;

    /// Applies an affine linear transform.
    fn affine(&self, affine: DAffine3) -> Tree;

    /// Tapers a shape to the given scale (in x/y) at the given height.
    fn taper(&self, scale: DVec2, height: f64) -> Tree;

    /// Shears a shape to the given offset (in x/y) at the given height.
    fn shear(&self, offset: DVec2, height: f64) -> Tree;
}

impl Transforms for Tree {
    fn translate(&self, translation: DVec3) -> Tree {
        self.affine(DAffine3::from_translation(translation))
    }

    fn rotate_x(&self, angle: f64) -> Tree {
        self.affine(DAffine3::from_rotation_x(angle))
    }

    fn rotate_y(&self, angle: f64) -> Tree {
        self.affine(DAffine3::from_rotation_y(angle))
    }

    fn rotate_z(&self, angle: f64) -> Tree {
        self.affine(DAffine3::from_rotation_z(angle))
    }

    fn affine(&self, affine: DAffine3) -> Tree {
        let point = Vec3::point();

        let matrix = affine.matrix3.inverse().transpose();
        let x_axis = Vec3::from_parameter(matrix.x_axis);
        let y_axis = Vec3::from_parameter(matrix.y_axis);
        let z_axis = Vec3::from_parameter(matrix.z_axis);

        let x = point.dot(x_axis) - affine.translation.x;
        let y = point.dot(y_axis) - affine.translation.y;
        let z = point.dot(z_axis) - affine.translation.z;

        self.remap_xyz(x, y, z)
    }

    fn taper(&self, scale: DVec2, height: f64) -> Tree {
        let point = Vec3::point();
        let alpha = point.z.clone() / height;

        let scale = alpha * Vec2::from_parameter(scale - DVec2::ONE) + 1.0.into();
        let x = point.x / scale.x;
        let y = point.y / scale.y;

        self.remap_xyz(x, y, point.z)
    }

    fn shear(&self, offset: DVec2, height: f64) -> Tree {
        let point = Vec3::point();
        let alpha = point.z.clone() / height;

        let offset = Vec2::from_parameter(offset);
        let offset = alpha * offset;
        let x = point.x - offset.x;
        let y = point.y - offset.y;

        self.remap_xyz(x, y, point.z)
    }
}
