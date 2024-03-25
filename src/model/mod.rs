mod config;
mod finger_cluster;
mod geometry;
mod key;
mod key_cluster;
mod key_positions;
mod primitives;
mod thumb_cluster;
mod util;

use std::path::Path;

use fidget::{context::IntoNode, Context};
use glam::{dvec3, DVec3};
use hex_color::HexColor;

use crate::viewer::{Component, MeshSettings, Viewable};

use config::Config;
use primitives::{BoundingBox, BoxShape};

pub use primitives::Shape;

pub struct Model {
    components: Vec<Component>,
    settings: MeshSettings,
    light_positions: Vec<DVec3>,
    background_color: HexColor,
}

impl Model {
    pub fn try_from_config(config_path: &Path) -> Result<Self, Error> {
        let config = Config::try_from_path(config_path)?;
        let size = 0.5;

        let mut context = Context::new();
        let root = BoxShape::new(dvec3(size, size, size)).into_node(&mut context)?;
        let shape = Shape::new(&context, root, BoundingBox::new(1.0, DVec3::ZERO))?;
        let components = vec![Component::new(shape, config.colors.keyboard)];

        let settings = MeshSettings {
            threads: 12,
            resolution: *config.preview.resolution,
        };

        Ok(Self {
            components,
            settings,
            light_positions: config.preview.light_positions,
            background_color: config.colors.background,
        })
    }
}

impl Viewable for Model {
    fn components(self) -> Vec<Component> {
        self.components
    }

    fn settings(&self) -> MeshSettings {
        self.settings
    }

    fn light_positions(&self) -> Vec<DVec3> {
        self.light_positions.clone()
    }

    fn background_color(&self) -> HexColor {
        self.background_color
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to parse config
    #[error("failed to parse config")]
    ParseConfig(#[from] config::Error),
    /// Failed to create model
    #[error("failed to create model")]
    CreateModel(#[from] fidget::Error),
}
