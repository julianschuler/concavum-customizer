use fidget::context::Tree;
use glam::DVec2;

use crate::{
    geometry::{rotate_90_degrees, Tangent},
    primitives::{Circle, Corner, Csg, IntoTree, Transforms},
};

/// A holder for a threaded M3 heat set insert.
pub struct InsertHolder {
    center: DVec2,
    edge1: DVec2,
    edge2: DVec2,
    index: usize,
}

impl InsertHolder {
    const INSERT_RADIUS: f64 = 2.0;
    const WALL_THICKNESS: f64 = 2.0;
    const HEIGHT: f64 = 7.0;
    const RADIUS: f64 = Self::INSERT_RADIUS + Self::WALL_THICKNESS;

    /// Creates a new insert holder from the given outline points and an index.
    pub fn from_outline_points(points: &[DVec2], index: usize, outline_offset: f64) -> Self {
        let n = points.len();
        let previous_point = points[(index + n - 1) % n];
        let point = points[index];
        let next_point = points[(index + 1) % n];

        let edge1 = rotate_90_degrees(previous_point - point).normalize();
        let edge2 = rotate_90_degrees(point - next_point).normalize();

        let outwards_direction = (edge1 + edge2).normalize();
        let center = point + (outline_offset - Self::RADIUS) * outwards_direction;

        Self {
            center,
            edge1,
            edge2,
            index,
        }
    }

    /// Returns the tangent to the insert holder along the first edge.
    pub fn tangent(&self) -> Tangent {
        let direction = self.edge1;
        let normal = -rotate_90_degrees(direction);
        let point = self.center + Self::RADIUS * normal;

        Tangent {
            point,
            direction,
            normal,
        }
    }

    /// Returns the center point of the insert.
    pub fn center(&self) -> DVec2 {
        self.center
    }

    /// Returns the outline vertex index of the insert holder.
    pub fn index(&self) -> usize {
        self.index
    }
}

impl From<InsertHolder> for Tree {
    fn from(insert_holder: InsertHolder) -> Self {
        let hole = Circle::new(InsertHolder::INSERT_RADIUS);

        Corner::new(insert_holder.edge1, insert_holder.edge2)
            .into_tree()
            .offset(InsertHolder::RADIUS)
            .difference(hole)
            .translate(insert_holder.center.extend(0.0))
            .extrude(0.0, InsertHolder::HEIGHT)
    }
}
