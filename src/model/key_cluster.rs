use glam::DAffine3;
use hex_color::HexColor;
use opencascade::primitives::Shape;

use crate::model::{
    config::{Config, PositiveDVec2},
    finger_cluster::FingerCluster,
    key_positions::KeyPositions,
    thumb_cluster::ThumbCluster,
    Component,
};

pub struct KeyCluster {
    shape: Shape,
    color: HexColor,
    key_positions: KeyPositions,
}

impl KeyCluster {
    pub fn from_config(config: &Config) -> Self {
        let key_distance: PositiveDVec2 = (&config.finger_cluster.key_distance).into();

        let key_positions = KeyPositions::from_config(config).tilt(config.keyboard.tilting_angle);

        let finger_cluster = FingerCluster::new(&key_positions.columns, &key_distance, config);
        let thumb_cluster = ThumbCluster::new(
            &key_positions.thumb_keys,
            &key_distance,
            *config.keyboard.circumference_distance,
        );

        let finger_cluster = finger_cluster.shape.subtract(&thumb_cluster.key_clearance);
        let thumb_cluster = thumb_cluster.mount;

        let shape = finger_cluster.union(&thumb_cluster).into();

        Self {
            shape,
            color: config.colors.keyboard,
            key_positions,
        }
    }

    pub fn finger_key_positions(&self) -> Vec<DAffine3> {
        self.key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter())
            .copied()
            .collect()
    }

    pub fn thumb_key_positions(&self) -> Vec<DAffine3> {
        self.key_positions.thumb_keys.to_owned()
    }
}

impl From<KeyCluster> for Component {
    fn from(cluster: KeyCluster) -> Self {
        Component::new(cluster.shape, cluster.color)
    }
}
