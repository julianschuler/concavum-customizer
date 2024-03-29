mod config;
mod finger_cluster;
mod geometry;
mod key;
mod key_cluster;
mod key_positions;
mod thumb_cluster;
mod util;

use std::path::Path;

use glam::DVec3;
use hex_color::HexColor;

pub use crate::viewer::model::{Component, Viewable};
use config::Config;
use key::Key;
use key_cluster::KeyCluster;

pub struct Model {
    components: Vec<Component>,
    light_positions: Vec<DVec3>,
    background_color: HexColor,
    triangulation_tolerance: f64,
}

impl Model {
    pub fn try_from_config(config_path: &Path) -> Result<Self, Error> {
        let config = Config::try_from_path(config_path)?;

        let mut components = Vec::new();

        let key_cluster = KeyCluster::from_config(&config);

        if config.preview.show_keys {
            let finger_key = Key::new(&config, 1.0);
            let thumb_key = Key::new(&config, 1.5);
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

        Ok(Self {
            components,
            light_positions: config.preview.light_positions,
            background_color: config.colors.background,
            triangulation_tolerance: *config.preview.triangulation_tolerance,
        })
    }
}

impl Viewable for Model {
    fn components(self) -> Vec<Component> {
        self.components
    }

    fn light_positions(&self) -> Vec<DVec3> {
        self.light_positions.clone()
    }

    fn background_color(&self) -> HexColor {
        self.background_color
    }

    fn triangulation_tolerance(&self) -> f64 {
        self.triangulation_tolerance
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to parse config
    #[error("failed to parse config")]
    ParseConfig(#[from] config::Error),
}
