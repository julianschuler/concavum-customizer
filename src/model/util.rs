use glam::{DVec2, DVec3};

use crate::model::config::{PLATE_X_2, PLATE_Y_2};

pub struct MountSize {
    pub width: f64,
    pub length: f64,
    pub height: f64,
}

impl MountSize {
    pub fn from_2d_points(points: &[DVec2], height: f64, circumference_distance: f64) -> Self {
        let height = height + f64::max(PLATE_X_2, PLATE_Y_2);

        let min_x = points
            .iter()
            .map(|point| point.x)
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let max_x = points
            .iter()
            .map(|point| point.x)
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let width = max_x - min_x + 2.0 * circumference_distance;

        let min_y = points
            .iter()
            .map(|point| point.y)
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let max_y = points
            .iter()
            .map(|point| point.y)
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let length = max_y - min_y + 2.0 * circumference_distance;

        Self {
            width,
            length,
            height,
        }
    }

    pub fn from_3d_points(points: &[DVec3], circumference_distance: f64) -> Self {
        let height = points
            .iter()
            .map(|point| point.z)
            .max_by(f64::total_cmp)
            .unwrap_or_default()
            + f64::max(PLATE_X_2, PLATE_Y_2);

        let min_x = points
            .iter()
            .map(|point| point.x)
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let max_x = points
            .iter()
            .map(|point| point.x)
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let width = max_x - min_x + 2.0 * circumference_distance;

        let min_y = points
            .iter()
            .map(|point| point.y)
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let max_y = points
            .iter()
            .map(|point| point.y)
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let length = max_y - min_y + 2.0 * circumference_distance;

        Self {
            width,
            length,
            height,
        }
    }
}
