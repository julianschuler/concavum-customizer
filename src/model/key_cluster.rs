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
        let circumference_distance = *config.keyboard.circumference_distance;

        let key_positions = KeyPositions::from_config(config).tilt(config.keyboard.tilting_angle);

        let finger_cluster = FingerCluster::new(
            &key_positions.columns,
            &key_distance,
            circumference_distance,
        );
        let thumb_cluster = ThumbCluster::new(
            &key_positions.thumb_keys,
            &key_distance,
            circumference_distance,
        );

        let finger_mount = finger_cluster.mount.subtract(&thumb_cluster.key_clearance);
        let thumb_mount = thumb_cluster.mount.subtract(&finger_cluster.key_clearance);

        let shape = finger_mount.union(&thumb_mount).into();

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
        Component::new(todo!(), todo!(), cluster.color)
    }
}
