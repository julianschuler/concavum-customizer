use std::ops::{Deref, DerefMut};

use crate::primitives::{Point, Position};

/// A path described by a list of points.
pub struct Path(Vec<Point>);

impl Path {
    /// Creates a new path from the given points.
    pub fn new(points: impl IntoIterator<Item = Point>) -> Self {
        Self(points.into_iter().collect())
    }

    /// Creates a new angled path from two points with the non-angled section in the center.
    pub fn angled_center(start: Point, end: Point) -> Self {
        let difference = end - start;
        let offset = difference.x.abs().min(difference.y.abs()) / 2.0 * difference.signum();

        Self(vec![start, start + offset, end - offset, end])
    }

    /// Creates a new angled path from two points with the non-angled section at the start.
    pub fn angled_start(start: Point, end: Point) -> Self {
        let difference = end - start;
        let offset = difference.x.abs().min(difference.y.abs()) * difference.signum();

        Self(vec![start, end - offset, end])
    }

    /// Creates a new angled path from two points with the non-angled section at the start.
    pub fn angled_end(start: Point, end: Point) -> Self {
        let difference = end - start;
        let offset = difference.x.abs().min(difference.y.abs()) * difference.signum();

        Self(vec![start, start + offset, end])
    }

    /// Returns the path relative to the given position.
    pub fn at(&self, position: Position) -> Self {
        Self(self.0.iter().map(|&point| position + point).collect())
    }
}

impl Deref for Path {
    type Target = Vec<Point>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
