mod config;
mod finger_cluster;
mod geometry;
mod key;
mod key_cluster;
mod key_positions;
mod thumb_cluster;
mod util;

use std::path::Path;

use fidget::{
    eval::types::Interval,
    mesh::{CellBounds, Settings},
    Context,
};
use glam::DVec3;
use hex_color::HexColor;

pub use crate::viewer::model::{Component, Viewable};
use config::Config;
use key::Key;
use key_cluster::KeyCluster;

pub struct Model {
    components: Vec<Component>,
    settings: Settings,
    light_positions: Vec<DVec3>,
    background_color: HexColor,
}

impl Model {
    pub fn try_from_config(config_path: &Path) -> Result<Self, Error> {
        const BOUND: f32 = 60.0;

        let config = Config::try_from_path(config_path)?;

        let components = vec![];

        let interval = Interval::new(-BOUND, BOUND);
        let bounds = CellBounds::new(interval, interval, interval);

        let settings = Settings {
            threads: 12,
            min_depth: 5,
            max_depth: 12,
            bounds,
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

    fn settings(&self) -> Settings {
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
