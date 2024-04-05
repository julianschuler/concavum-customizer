mod finger_cluster;
mod geometry;
mod key;
mod key_cluster;
mod key_positions;
mod primitives;
mod thumb_cluster;
mod util;

use glam::DVec3;
use hex_color::HexColor;

use crate::{
    config::{Config, Error as ConfigError},
    viewer::{Component, MeshSettings, Viewable},
};
pub use primitives::Shape;

use key::Key;
use key_cluster::KeyCluster;

pub struct Model {
    components: Vec<Component>,
    settings: MeshSettings,
    light_positions: Vec<DVec3>,
    background_color: HexColor,
}

impl Model {
    pub fn try_from_config(config: Config) -> Result<Self, Error> {
        let mut components = Vec::new();
        let key_cluster = KeyCluster::from_config(&config)?;

        if config.preview.show_keys {
            let finger_key = Key::new(&config, 1.0)?;
            let thumb_key = Key::new(&config, 1.5)?;
            let (mut finger_keycap, mut finger_switch) = finger_key.into();
            let (mut thumb_keycap, mut thumb_switch) = thumb_key.into();

            let finger_key_positions = key_cluster.finger_key_positions();
            let thumb_key_positions = key_cluster.thumb_key_positions();

            finger_keycap.with_positions(finger_key_positions.clone());
            finger_switch.with_positions(finger_key_positions);
            thumb_keycap.with_positions(thumb_key_positions.clone());
            thumb_switch.with_positions(thumb_key_positions);

            components.push(finger_keycap);
            components.push(finger_switch);
            components.push(thumb_keycap);
            components.push(thumb_switch);
        }

        components.push(key_cluster.into());

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
    ParseConfig(#[from] ConfigError),
    /// Failed to create model
    #[error("failed to create model")]
    CreateModel(#[from] fidget::Error),
}
