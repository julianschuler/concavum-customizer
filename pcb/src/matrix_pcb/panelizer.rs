use model::matrix_pcb::PAD_SIZE;

use crate::{
    matrix_pcb::features::{Column, Features},
    point,
    primitives::Point,
};

/// A bounding box for the matrix PCB.
struct BoundingBox {
    min: Point,
    max: Point,
}

impl BoundingBox {
    /// Creates a new bounding box from the given matrix PCB features.
    pub fn from_features(features: &Features) -> Self {
        let points = features
            .columns
            .iter()
            .flat_map(Column::positions)
            .chain(features.thumb_switches.positions())
            .flat_map(|&position| {
                [
                    position + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
                    position + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
                    position + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
                    position + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
                ]
            });

        Self::from_points(points)
    }

    /// Creates a new bounding box from the given points.
    pub fn from_points(points: impl IntoIterator<Item = Point>) -> Self {
        let (min, max) = points
            .into_iter()
            .fold((Point::MAX, Point::MIN), |(min, max), point| {
                (min.min(point), max.max(point))
            });

        Self { min, max }
    }
}
