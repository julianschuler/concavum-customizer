//! The `model` crate contains everything related to creating a keyboard model from a given configuration.

mod geometry;
mod interface_pcb;
mod key_positions;
mod keyboard;
mod primitives;
mod util;

use config::{Colors, Config, Preview};

use keyboard::Keyboard;

pub use fidget::mesh::{Mesh, Settings as MeshSettings};

/// A set of settings used for displaying a model.
#[derive(Clone, Default)]
pub struct DisplaySettings {
    /// The preview settings of the model.
    pub preview: Preview,
    /// The colors of the model.
    pub colors: Colors,
}

impl From<Config> for DisplaySettings {
    fn from(config: Config) -> Self {
        DisplaySettings {
            preview: config.preview,
            colors: config.colors,
        }
    }
}

/// A keyboard model with colors and preview settings.
pub struct Model {
    /// The keyboard model.
    pub keyboard: Keyboard,
    /// The resolution used for meshing.
    pub resolution: f64,
    /// The settings used for displaying the model.
    pub display_settings: DisplaySettings,
}

impl Model {
    /// Creates a new model from a given configuration.
    #[must_use]
    pub fn from_config(config: Config) -> Self {
        let keyboard = Keyboard::from_config(&config);

        Self {
            keyboard,
            resolution: config.keyboard.resolution.into(),
            display_settings: config.into(),
        }
    }
}
