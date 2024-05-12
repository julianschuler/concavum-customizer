mod columns;
mod thumb_keys;

use std::ops::Mul;

use glam::{dvec3, DAffine3};

use crate::config::Config;
use crate::model::keyboard::Bounds;

pub use columns::{Column, ColumnType, Columns};
pub use thumb_keys::ThumbKeys;

pub struct KeyPositions {
    pub columns: Columns,
    pub thumb_keys: ThumbKeys,
}

impl KeyPositions {
    pub fn from_config(config: &Config) -> Self {
        const CENTER_OFFSET: f64 = 10.0;
        const Z_OFFSET: f64 = 12.0;

        let columns = Columns::from_config(&config.finger_cluster);
        let thumb_keys = ThumbKeys::from_config(&config.thumb_cluster);

        let (tilting_x, tilting_y) = (
            config.keyboard.tilting_angle.x.to_radians(),
            config.keyboard.tilting_angle.y.to_radians(),
        );
        let tilted_positions = (DAffine3::from_rotation_y(tilting_y)
            * DAffine3::from_rotation_x(tilting_x))
            * Self {
                columns,
                thumb_keys,
            };

        let z_offset = Z_OFFSET
            - tilted_positions
                .columns
                .min_z()
                .min(tilted_positions.thumb_keys.min_z());

        let column_bounds = Bounds::from_outline_points_and_height(
            &tilted_positions.columns.outline_points(),
            0.0,
            *config.keyboard.circumference_distance,
        );
        let thumb_key_bounds = Bounds::from_outline_points_and_height(
            &tilted_positions.thumb_keys.outline_points(),
            0.0,
            *config.keyboard.circumference_distance,
        );
        let bounds = column_bounds.union(&thumb_key_bounds);

        DAffine3::from_translation(dvec3(CENTER_OFFSET - bounds.min.x, 0.0, z_offset))
            * tilted_positions
    }
}

impl Mul<KeyPositions> for DAffine3 {
    type Output = KeyPositions;

    fn mul(self, key_positions: KeyPositions) -> KeyPositions {
        KeyPositions {
            columns: self * key_positions.columns,
            thumb_keys: self * key_positions.thumb_keys,
        }
    }
}
