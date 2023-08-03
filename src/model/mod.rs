mod config;
mod finger_cluster;
mod helper;
mod key;

use std::path::Path;

use glam::DVec3;
use hex_color::HexColor;

pub use crate::viewer::model::{Component, ViewableModel};
use config::Config;
use key::Key;

use self::finger_cluster::KeyPositions;

pub struct Model {
    components: Vec<Component>,
    light_directions: Vec<DVec3>,
    background_color: HexColor,
    triangulation_tolerance: f64,
}

impl Model {
    pub fn try_from_config(config_path: &Path) -> Result<Self, Error> {
        let config = Config::try_from_path(config_path)?;

        let mut components = Vec::new();

        let key = Key::new(&config, 1.0);
        let (mut keycap, mut switch) = key.into();

        let key_positions = KeyPositions::from_config(&config.finger_cluster);
        let key_positions: Vec<_> = key_positions.positions.into_iter().flatten().collect();

        keycap.with_positions(key_positions.clone());
        switch.with_positions(key_positions);

        components.push(keycap);
        components.push(switch);

        Ok(Self {
            components,
            light_directions: config.preview.light_directions,
            background_color: config.colors.background,
            triangulation_tolerance: *config.preview.triangulation_tolerance,
        })
    }
}

impl ViewableModel for Model {
    fn components(self) -> Vec<Component> {
        self.components
    }

    fn light_directions(&self) -> Vec<DVec3> {
        self.light_directions.clone()
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
