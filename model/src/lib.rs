//! The model crate contains everything related to creating a keyboard model from a given configuration.

mod geometry;
mod interface_pcb;
mod key_positions;
mod keyboard;
mod primitives;
mod util;

use config::{Colors, Config, Preview};

use keyboard::Keyboard;

pub use fidget::mesh::{Mesh, Settings as MeshSettings};

pub struct Model {
    pub keyboard: Keyboard,
    pub colors: Colors,
    pub settings: Preview,
}

impl Model {
    #[must_use]
    pub fn from_config(config: Config) -> Self {
        let keyboard = Keyboard::from_config(&config);

        Self {
            keyboard,
            colors: config.colors,
            settings: config.preview,
        }
    }
}
