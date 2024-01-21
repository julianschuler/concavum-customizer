use glam::{DAffine3, DVec2};

use crate::model::config::{PLATE_X_2, PLATE_Y_2};

pub struct MountSize {
    pub width: f64,
    pub length: f64,
    pub height: f64,
}

impl MountSize {
    fn calculate_height<'a>(points: impl IntoIterator<Item = &'a DAffine3>) -> f64 {
        points
            .into_iter()
            .map(|position| position.translation.z)
            .max_by(f64::total_cmp)
            .unwrap_or_default()
            + PLATE_X_2
            + PLATE_Y_2
    }

    pub fn from_points_and_positions<'a>(
        points: impl IntoIterator<Item = DVec2>,
        positions: impl IntoIterator<Item = &'a DAffine3>,
        circumference_distance: f64,
    ) -> Self {
        let (min_x, min_y, max_x, max_y) = points.into_iter().fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, min_y, max_x, max_y), point| {
                (
                    min_x.min(point.x),
                    min_y.min(point.y),
                    max_x.max(point.x),
                    max_y.max(point.y),
                )
            },
        );

        let width = max_x - min_x + 2.0 * circumference_distance;
        let length = max_y - min_y + 2.0 * circumference_distance;
        let height = Self::calculate_height(positions);

        Self {
            width,
            length,
            height,
        }
    }
}
