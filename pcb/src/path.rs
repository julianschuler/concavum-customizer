use std::ops::{Deref, DerefMut};

use glam::{vec2, Vec2};

use crate::{
    primitives::{Point, Position},
    unit::Length,
};

/// A directed path described by a list of points.
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

    /// Creates a chamfered path with a chamfer with the given depth.
    pub fn chamfered(start: Point, end: Point, depth: Length, right: bool) -> Self {
        let difference = end - start;

        if (difference.x.signum() == difference.y.signum()) == right {
            Self(vec![
                start,
                Point::new(start.x(), end.y() - difference.y.signum() * depth),
                Point::new(start.x() + difference.x.signum() * depth, end.y()),
                end,
            ])
        } else {
            Self(vec![
                start,
                Point::new(end.x() - difference.x.signum() * depth, start.y()),
                Point::new(end.x(), start.y() + difference.y.signum() * depth),
                end,
            ])
        }
    }

    /// Returns the path relative to the given position.
    pub fn at(&self, position: Position) -> Self {
        Self(self.iter().map(|&point| position + point).collect())
    }

    /// Offsets the path by the given value.
    pub fn offset(&self, offset: f32) -> Self {
        let mut offset_path = Vec::new();

        if let Some(&[first_point, second_point]) = self.first_chunk() {
            let offset_direction =
                offset * rotate_90_degrees(second_point - first_point).normalize();
            offset_path.push(first_point + offset_direction);
        }

        for window in self.windows(3) {
            let previous_point = window[0];
            let point = window[1];
            let next_point = window[2];

            let previous_line_direction = point - previous_point;
            let next_line_direction = next_point - point;

            let previous_line_point =
                point + offset * rotate_90_degrees(previous_line_direction).normalize();
            let next_line_point =
                point + offset * rotate_90_degrees(next_line_direction).normalize();

            let previous_line = Line::new(previous_line_point, previous_line_direction);
            let next_line = Line::new(next_line_point, next_line_direction);

            if let Some(intersection) = previous_line.intersection(&next_line) {
                offset_path.push(intersection);
            }
        }

        if let Some(&[second_to_last_point, last_point]) = self.last_chunk() {
            let offset_direction =
                offset * rotate_90_degrees(last_point - second_to_last_point).normalize();
            offset_path.push(last_point + offset_direction);
        }

        Self(offset_path)
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

/// A line in 2D space.
struct Line {
    point: Point,
    direction: Vec2,
}

impl Line {
    /// Creates a new line from the given point and direction.
    fn new(point: Point, direction: Vec2) -> Self {
        Self { point, direction }
    }

    /// Computes the intersection between `self` and `other`, if present.
    fn intersection(&self, other: &Self) -> Option<Point> {
        let normal = rotate_90_degrees(self.direction);
        let intersection_parameter =
            normal.dot(self.point - other.point) / normal.dot(other.direction);

        intersection_parameter
            .is_finite()
            .then(|| other.point + intersection_parameter * other.direction)
    }
}

/// Rotates a vector 90 degrees counterclockwise.
fn rotate_90_degrees(vector: Vec2) -> Vec2 {
    vec2(vector.y, -vector.x)
}
