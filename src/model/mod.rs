mod geometry;
mod interface_pcb;
mod key_positions;
mod keyboard;
mod primitives;
mod util;

use crate::config::{Colors, Config, Preview};

use keyboard::Keyboard;

pub struct Model {
    pub keyboard: Keyboard,
    pub colors: Colors,
    pub settings: Preview,
}

impl Model {
    pub fn from_config(config: Config) -> Self {
        let keyboard = Keyboard::from_config(&config);

        Self {
            keyboard,
            colors: config.colors,
            settings: config.preview,
        }
    }
}
