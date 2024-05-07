use fidget::context::Tree;
use glam::{DAffine3, DVec2, DVec3};

use crate::{
    config::EPSILON,
    model::primitives::vector::{Vec2, Vec3, Vector},
};

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

/// A trait defining rounded Constructive Solid Geometry (CSG) operations.
pub trait RoundedCsg {
    /// Performs the union between two shapes while rounding the new edges.
    fn rounded_union<T: Into<Tree>>(&self, other: T, radius: f64) -> Self;

    /// Performs the difference between two shapes while rounding the new edges.
    fn rounded_difference<T: Into<Tree>>(&self, other: T, radius: f64) -> Self;

    /// Performs the intersection between two shapes while rounding the new edges.
    fn rounded_intersection<T: Into<Tree>>(&self, other: T, radius: f64) -> Self;
}

impl RoundedCsg for Tree {
    fn rounded_union<T: Into<Tree>>(&self, other: T, radius: f64) -> Self {
        self.neg()
            .rounded_intersection(other.into().neg(), radius)
            .neg()
    }

    fn rounded_difference<T: Into<Tree>>(&self, other: T, radius: f64) -> Self {
        self.rounded_intersection(other.into().neg(), radius)
    }

    fn rounded_intersection<T: Into<Tree>>(&self, other: T, radius: f64) -> Self {
        let vector = Vec2 {
            x: self.clone() + radius,
            y: other.into() + radius,
        };

        let outer = vector.max(EPSILON.into()).length();
        let inner = vector.max_elem().min(0.0);

        outer + inner - radius
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
        let point = Vec3::point() - affine.translation.into();

        let matrix = affine.matrix3.inverse().transpose();
        let x = point.dot(matrix.x_axis.into());
        let y = point.dot(matrix.y_axis.into());
        let z = point.dot(matrix.z_axis.into());

        self.remap_xyz(x, y, z)
    }

    fn taper(&self, scale: DVec2, height: f64) -> Tree {
        let point = Vec3::point();
        let alpha = point.z.clone() / height;

        let scale = alpha * Vec2::from(scale - DVec2::ONE) + 1.0.into();
        let x = point.x / scale.x;
        let y = point.y / scale.y;

        self.remap_xyz(x, y, point.z)
    }

    fn shear(&self, offset: DVec2, height: f64) -> Tree {
        let point = Vec3::point();
        let alpha = point.z.clone() / height;

        let offset = alpha * Vec2::from(offset);
        let x = point.x - offset.x;
        let y = point.y - offset.y;

        self.remap_xyz(x, y, point.z)
    }
}
