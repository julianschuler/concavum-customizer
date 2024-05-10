mod cluster_bounds;
mod finger_cluster;
mod geometry;
mod insert_holder;
mod key_cluster;
mod key_positions;
mod primitives;
mod thumb_cluster;
mod util;

use crate::config::{Colors, Config, Preview};
pub use primitives::Shape;

use key_cluster::KeyCluster;
use key_positions::KeyPositions;

pub struct Model {
    pub keyboard: Shape,
    pub key_positions: KeyPositions,
    pub colors: Colors,
    pub settings: Preview,
}

impl Model {
    pub fn from_config(config: Config) -> Self {
        let key_cluster = KeyCluster::from_config(&config);

        Self {
            keyboard: key_cluster.shape,
            key_positions: key_cluster.key_positions,
            colors: config.colors,
            settings: config.preview,
        }
    }
}
