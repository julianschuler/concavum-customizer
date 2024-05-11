mod geometry;
mod key_positions;
mod keyboard;
mod primitives;
mod util;

use crate::config::{Colors, Config, Preview};

use key_positions::KeyPositions;
use keyboard::Keyboard;

pub use primitives::Shape;

pub struct Model {
    pub keyboard: Shape,
    pub key_positions: KeyPositions,
    pub colors: Colors,
    pub settings: Preview,
}

impl Model {
    pub fn from_config(config: Config) -> Self {
        let keyboard = Keyboard::from_config(&config);

        Self {
            keyboard: keyboard.shape,
            key_positions: keyboard.key_positions,
            colors: config.colors,
            settings: config.preview,
        }
    }
}
