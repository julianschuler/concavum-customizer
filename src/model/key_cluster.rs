use fidget::Context;
use glam::{DAffine3, DVec3};
use hex_color::HexColor;

use crate::model::{
    config::{Config, PositiveDVec2},
    finger_cluster::FingerCluster,
    key_positions::KeyPositions,
    primitives::{Bounds, Csg, Result, Shape},
    thumb_cluster::ThumbCluster,
    Component,
};

pub struct KeyCluster {
    shape: Shape,
    color: HexColor,
    key_positions: KeyPositions,
}

impl KeyCluster {
    pub fn from_config(config: &Config) -> Result<Self> {
        let key_distance: PositiveDVec2 = (&config.finger_cluster.key_distance).into();
        let circumference_distance = *config.keyboard.circumference_distance;
        let key_positions = KeyPositions::from_config(config).tilt(config.keyboard.tilting_angle);

        let mut context = Context::new();

        let finger_cluster = FingerCluster::new(
            &mut context,
            &key_positions.columns,
            &key_distance,
            circumference_distance,
        )?;
        let thumb_cluster = ThumbCluster::new(
            &mut context,
            &key_positions.thumb_keys,
            &key_distance,
            circumference_distance,
        )?;

        let finger_mount = context.difference(finger_cluster.mount, thumb_cluster.key_clearance)?;
        let thumb_mount = context.difference(thumb_cluster.mount, finger_cluster.key_clearance)?;
        let cluster = context.union(finger_mount, thumb_mount)?;

        let shape = Shape::new(&context, cluster, Bounds::new(200.0, DVec3::ZERO))?;

        Ok(Self {
            shape,
            color: config.colors.keyboard,
            key_positions,
        })
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
