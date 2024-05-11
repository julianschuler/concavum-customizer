use fidget::context::Tree;
use glam::{dvec3, DVec3};

use crate::{
    config::Config,
    model::{
        finger_cluster::FingerCluster,
        geometry::Plane,
        key_positions::KeyPositions,
        primitives::{BoxShape, Csg, HalfSpace, RoundedCsg, Shape, Transforms},
        thumb_cluster::ThumbCluster,
    },
};

pub struct KeyCluster {
    pub shape: Shape,
    pub key_positions: KeyPositions,
}

impl KeyCluster {
    pub fn from_config(config: &Config) -> Self {
        let key_positions = KeyPositions::from_config(config);

        let finger_cluster = FingerCluster::new(&key_positions.columns, &config.keyboard);
        let thumb_cluster = ThumbCluster::new(&key_positions.thumb_keys, &config.keyboard);
        let bounds = finger_cluster.bounds.union(&thumb_cluster.bounds);
        let inserts = finger_cluster
            .insert_holders
            .union(thumb_cluster.insert_holder);

        // Subtract key clearances from each other and combine the clusters
        let rounding_radius = config.keyboard.rounding_radius;
        let finger_key_clearance = finger_cluster.key_clearance;
        let finger_cluster = finger_cluster
            .cluster
            .rounded_difference(thumb_cluster.key_clearance, rounding_radius);
        let thumb_cluster = thumb_cluster
            .cluster
            .rounded_difference(finger_key_clearance, rounding_radius);
        let combined_cluster = finger_cluster.union(thumb_cluster);

        // Hollow out the combined cluster and cut off everthing below a z value of 0
        let half_space = HalfSpace::new(Plane::new(DVec3::ZERO, DVec3::NEG_Z));
        let hollowed_cluster = combined_cluster.shell(*config.keyboard.shell_thickness);
        let cluster = hollowed_cluster.intersection(half_space);

        // Add the insert holders and cutouts
        let cluster = cluster.union(inserts);
        let cluster = cluster.difference(Self::key_cutouts(&key_positions));

        let shape = Shape::new(&cluster, bounds.into());

        Self {
            shape,
            key_positions,
        }
    }

    fn key_cutouts(key_positions: &KeyPositions) -> Tree {
        let key_cutout: Tree = BoxShape::new(dvec3(14.0, 14.0, 10.0)).into();

        key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter())
            .chain(key_positions.thumb_keys.iter())
            .map(|&position| key_cutout.affine(position))
            .reduce(|a, b| a.union(b))
            .expect("there is more than one key")
    }
}
