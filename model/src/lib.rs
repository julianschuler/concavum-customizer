//! The `model` crate contains everything related to creating a keyboard model from a given configuration.

mod geometry;
mod interface_pcb;
mod key_positions;
mod keyboard;
mod primitives;
mod util;

/// The matrix PCB module cointans everything related to creating a model of the matrix PCB.
pub mod matrix_pcb;

use config::{Colors, Config, Preview};

use key_positions::KeyPositions;
use keyboard::Keyboard;
use matrix_pcb::MatrixPcb;

pub use fidget::mesh::{Mesh, Settings as MeshSettings};
pub use primitives::Bounds;

/// A set of settings used for displaying a model.
#[derive(Clone, Default)]
pub struct DisplaySettings {
    /// The preview settings of the model.
    pub preview: Preview,
    /// The colors of the model.
    pub colors: Colors,
}

impl From<&Config> for DisplaySettings {
    fn from(config: &Config) -> Self {
        DisplaySettings {
            preview: config.preview.clone(),
            colors: config.colors.clone(),
        }
    }
}

/// A keyboard model with colors and preview settings.
pub struct Model {
    /// The keyboard model.
    pub keyboard: Keyboard,
    /// The matrix PCB.
    pub matrix_pcb: MatrixPcb,
    /// The position of the keys.
    pub key_positions: KeyPositions,
    /// The resolution used for meshing.
    pub resolution: f64,
    /// The settings used for displaying the model.
    pub display_settings: DisplaySettings,
}

impl Model {
    /// Creates a new model from a given configuration.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let key_positions = KeyPositions::from_config(config);
        let keyboard = Keyboard::new(&key_positions, &config.keyboard);
        let matrix_pcb = MatrixPcb::from_positions(&key_positions);

        Self {
            keyboard,
            matrix_pcb,
            key_positions,
            resolution: config.keyboard.resolution.into(),
            display_settings: config.into(),
        }
    }
}
