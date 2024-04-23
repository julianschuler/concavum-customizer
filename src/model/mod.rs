mod finger_cluster;
mod geometry;
mod key_cluster;
mod key_positions;
mod primitives;
mod thumb_cluster;
mod util;

use glam::DVec3;

use crate::config::{Colors, Config};
pub use primitives::Shape;

use key_cluster::KeyCluster;
use key_positions::KeyPositions;

pub struct Model {
    pub keyboard: Shape,
    pub key_positions: KeyPositions,
    pub light_positions: Vec<DVec3>,
    pub resolution: f64,
    pub colors: Colors,
}

impl Model {
    pub fn from_config(config: Config) -> Self {
        let key_cluster = KeyCluster::from_config(&config);

        Self {
            keyboard: key_cluster.shape,
            key_positions: key_cluster.key_positions,
            light_positions: config.preview.light_positions,
            resolution: *config.preview.resolution,
            colors: config.colors,
        }
    }
}
