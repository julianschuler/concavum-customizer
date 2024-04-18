use glam::DVec3;

use crate::{
    config::{Config, PositiveDVec2},
    model::{
        finger_cluster::FingerCluster,
        geometry::Plane,
        key_positions::KeyPositions,
        primitives::{Bounds, Csg, HalfSpace, Result, Shape},
        thumb_cluster::ThumbCluster,
    },
};

pub struct KeyCluster {
    pub shape: Shape,
    pub key_positions: KeyPositions,
}

impl KeyCluster {
    pub fn from_config(config: &Config) -> Result<Self> {
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

        // Subtract key clearances from each other and combine the mounts
        let finger_mount = finger_cluster.mount.difference(thumb_cluster.key_clearance);
        let thumb_mount = thumb_cluster.mount.difference(finger_cluster.key_clearance);
        let combined_mount = finger_mount.union(thumb_mount);

        // Hollow out the combined mount and cut off everthing below a z value of 0
        let half_space = HalfSpace::new(Plane::new(DVec3::ZERO, DVec3::NEG_Z));
        let hollowed_cluster = combined_mount.shell(*config.keyboard.shell_thickness);
        let cluster = hollowed_cluster.intersection(half_space);

        let shape = Shape::new(&cluster, Bounds::new(200.0, DVec3::ZERO))?;

        Ok(Self {
            shape,
            key_positions,
        })
    }
}
