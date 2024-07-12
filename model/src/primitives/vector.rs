use std::ops::{Add, Mul, Sub};

use fidget::context::Tree;
use glam::{DVec2, DVec3};

/// A trait defining operations on a vector.
#[allow(dead_code)]
pub trait Vector: Sized {
    /// Applies a unary function element-wise.
    fn map_unary<F>(&self, f: F) -> Self
    where
        F: Fn(&Tree) -> Tree,
        Self: Sized;

    /// Applies a binary function element-wise.
    fn map_binary<F>(&self, other: Self, f: F) -> Self
    where
        F: Fn(&Tree, Tree) -> Tree;

    /// Folds all Vector elements using a binary function.
    fn fold<F>(&self, f: F) -> Tree
    where
        F: Fn(&Tree, Tree) -> Tree;

    /// Calculates the element-wise absolute value.
    fn abs(&self) -> Self {
        self.map_unary(Tree::abs)
    }

    /// Squares each element of a Vector.
    fn square(&self) -> Self {
        self.map_unary(Tree::square)
    }

    /// Calculates the element-wise mininum.
    fn min(&self, other: Self) -> Self {
        self.map_binary(other, Tree::min)
    }

    /// Calculates the element-wise maximum.
    fn max(&self, other: Self) -> Self {
        self.map_binary(other, Tree::max)
    }

    /// Calculates the minimum value of all elements.
    fn min_elem(&self) -> Tree {
        self.fold(Tree::min)
    }

    /// Calculates the maximum value of all elements.
    fn max_elem(&self) -> Tree {
        self.fold(Tree::max)
    }

    /// Calculates the sum over all elements.
    fn sum(&self) -> Tree {
        self.fold(|a, b| a.clone() + b)
    }

    /// Calculates the dot product of two Vectors.
    fn dot(&self, other: Self) -> Tree {
        let product = self.map_binary(other, |a, b| a.clone() * b);
        product.sum()
    }

    /// Calculates the euclidean norm of a Vector.
    fn length(&self) -> Tree {
        self.squared_length().sqrt()
    }

    /// Calculates the squared euclidean length of a Vector.
    fn squared_length(&self) -> Tree {
        self.square().sum()
    }
}

/// A 3-dimensional vector of nodes.
#[derive(Clone)]
pub struct Vec3 {
    pub x: Tree,
    pub y: Tree,
    pub z: Tree,
}

impl Vec3 {
    /// Creates a vector from the x, y and z variables.
    pub fn point() -> Self {
        let x = Tree::x();
        let y = Tree::y();
        let z = Tree::z();

        Self { x, y, z }
    }
}

impl From<f64> for Vec3 {
    fn from(value: f64) -> Self {
        Self {
            x: value.into(),
            y: value.into(),
            z: value.into(),
        }
    }
}

impl From<DVec3> for Vec3 {
    fn from(vector: DVec3) -> Self {
        let x = Tree::constant(vector.x);
        let y = Tree::constant(vector.y);
        let z = Tree::constant(vector.z);

        Self { x, y, z }
    }
}

impl Vector for Vec3 {
    fn map_unary<F>(&self, f: F) -> Self
    where
        F: Fn(&Tree) -> Tree,
    {
        Self {
            x: f(&self.x),
            y: f(&self.y),
            z: f(&self.z),
        }
    }

    fn map_binary<F>(&self, other: Self, f: F) -> Self
    where
        F: Fn(&Tree, Tree) -> Tree,
    {
        Self {
            x: f(&self.x, other.x),
            y: f(&self.y, other.y),
            z: f(&self.z, other.z),
        }
    }

    fn fold<F>(&self, f: F) -> Tree
    where
        F: Fn(&Tree, Tree) -> Tree,
    {
        f(&f(&self.x, self.y.clone()), self.z.clone())
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x = self.x + rhs.x;
        let y = self.y + rhs.y;
        let z = self.z + rhs.z;

        Self { x, y, z }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x = self.x - rhs.x;
        let y = self.y - rhs.y;
        let z = self.z - rhs.z;

        Self { x, y, z }
    }
}

impl Mul<Vec3> for Tree {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let x = self.clone() * rhs.x;
        let y = self.clone() * rhs.y;
        let z = self * rhs.z;

        Self::Output { x, y, z }
    }
}

/// A 2-dimensional vector of nodes.
#[derive(Clone)]
pub struct Vec2 {
    pub x: Tree,
    pub y: Tree,
}

impl Vec2 {
    /// Creates a vector from the x and y variables.
    pub fn point() -> Self {
        let x = Tree::x();
        let y = Tree::y();

        Self { x, y }
    }
}

impl From<f64> for Vec2 {
    fn from(value: f64) -> Self {
        Self {
            x: value.into(),
            y: value.into(),
        }
    }
}

impl From<DVec2> for Vec2 {
    fn from(vector: DVec2) -> Self {
        let x = Tree::constant(vector.x);
        let y = Tree::constant(vector.y);

        Self { x, y }
    }
}

impl Vector for Vec2 {
    fn map_unary<F>(&self, f: F) -> Self
    where
        F: Fn(&Tree) -> Tree,
    {
        Self {
            x: f(&self.x),
            y: f(&self.y),
        }
    }

    fn map_binary<F>(&self, other: Self, f: F) -> Self
    where
        F: Fn(&Tree, Tree) -> Tree,
    {
        Self {
            x: f(&self.x, other.x),
            y: f(&self.y, other.y),
        }
    }

    fn fold<F>(&self, f: F) -> Tree
    where
        F: Fn(&Tree, Tree) -> Tree,
    {
        f(&self.x, self.y.clone())
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x = self.x + rhs.x;
        let y = self.y + rhs.y;

        Self { x, y }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x = self.x - rhs.x;
        let y = self.y - rhs.y;

        Self { x, y }
    }
}

impl Mul<Vec2> for Tree {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        let x = self.clone() * rhs.x;
        let y = self * rhs.y;

        Self::Output { x, y }
    }
}
