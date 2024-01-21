use glam::{dvec3, DAffine3};

use crate::model::config::{PLATE_X_2, PLATE_Y_2};

/// Upper bound for the size of a mount
pub struct MountSize {
    pub width: f64,
    pub length: f64,
    pub height: f64,
}

impl MountSize {
    pub fn from_positions<'a>(
        positions: impl IntoIterator<Item = &'a DAffine3>,
        circumference_distance: f64,
    ) -> Self {
        const PADDING: f64 = PLATE_X_2 + PLATE_Y_2;

        let (min, max) = positions.into_iter().fold(
            (
                dvec3(f64::INFINITY, f64::INFINITY, f64::INFINITY),
                dvec3(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            ),
            |(min, max), point| (min.min(point.translation), max.max(point.translation)),
        );

        let width = max.x - min.x + 2.0 * circumference_distance + PADDING;
        let length = max.y - min.y + 2.0 * circumference_distance + PADDING;
        let height = max.z + PADDING;

        Self {
            width,
            length,
            height,
        }
    }
}
