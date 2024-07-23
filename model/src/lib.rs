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

/// A keyboard model with colors and preview settings.
pub struct Model {
    /// The keyboard model.
    pub keyboard: Keyboard,
    /// Colors used for displaying the model.
    pub colors: Colors,
    /// Preview settings used for displaying the model.
    pub settings: Preview,
}

impl Model {
    /// Creates a new model from a given configuration.
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
