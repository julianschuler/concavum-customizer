use glam::{dvec2, DAffine3};
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
        const KEY_CLEARANCE: f64 = 1.0;

        let key_distance: PositiveDVec2 = (&config.finger_cluster.key_distance).into();
        let key_clearance = dvec2(
            key_distance.x + KEY_CLEARANCE,
            key_distance.y + KEY_CLEARANCE,
        );

        let key_positions = KeyPositions::from_config(config).tilt(config.keyboard.tilting_angle);

        let finger_cluster = FingerCluster::new(&key_positions.columns, &key_clearance, config);
        let thumb_cluster = ThumbCluster::new(
            &key_positions.thumb_keys,
            *config.keyboard.circumference_distance,
        );

        let shape = finger_cluster.shape.union(&thumb_cluster.shape).into();

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
