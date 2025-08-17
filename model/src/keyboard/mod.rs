mod bottom_plate;
mod finger_cluster;
mod insert_holder;
mod thumb_cluster;

use std::iter::once;

use config::Keyboard as Config;
use fidget::context::Tree;
use glam::{dvec2, DAffine3, DVec2, DVec3};

pub use insert_holder::InsertHolder;

use crate::{
    geometry::{vec_z, Plane},
    key_positions::KeyPositions,
    primitives::{BoxShape, Csg, HalfSpace, IntoTree, RoundedCsg, Shape, Transforms, EPSILON},
};

use bottom_plate::BottomPlate;
use finger_cluster::FingerCluster;
use thumb_cluster::ThumbCluster;

/// A keyboard.
pub struct Keyboard {
    /// The case of the keyboard.
    pub case: Shape,
    /// The bottom plate of the keyboard.
    pub bottom_plate: Shape,
    /// A simplified preview shape of the keyboard.
    pub preview: Shape,
    /// The position of the interface PCB.
    pub interface_pcb_position: DAffine3,
}

impl Keyboard {
    /// Creates a keyboard from the given key positions and configuration.
    pub fn new(key_positions: &KeyPositions, config: &Config) -> Self {
        let shell_thickness = config.shell_thickness.into();
        let finger_cluster = FingerCluster::new(&key_positions.columns, config);
        let thumb_cluster = ThumbCluster::new(&key_positions.thumb_keys, config);

        let bounds = finger_cluster.bounds.union(thumb_cluster.bounds);
        let interface_pcb = finger_cluster.interface_pcb;
        let insert_holders: Vec<_> = finger_cluster
            .insert_holders
            .into_iter()
            .chain(once(thumb_cluster.insert_holder))
            .collect();
        let cluster_outline = finger_cluster.outline.union(thumb_cluster.outline);

        // Subtract key clearances from each other and combine the clusters
        let rounding_radius = config.rounding_radius.into();
        let finger_key_clearance = finger_cluster.key_clearance;
        let finger_cluster = finger_cluster
            .cluster
            .rounded_difference(thumb_cluster.key_clearance, rounding_radius);
        let thumb_cluster = thumb_cluster
            .cluster
            .rounded_difference(finger_key_clearance, rounding_radius);
        let combined_cluster = finger_cluster.union(thumb_cluster);

        // Hollow out the combined cluster and cut off everything below a Z value of 0
        let half_space = HalfSpace::new(Plane::new(DVec3::ZERO, DVec3::NEG_Z)).into_tree();
        let hollowed_cluster = combined_cluster.shell(shell_thickness);
        let cluster = hollowed_cluster.intersection(half_space.clone());

        // Calculate the bottom plate and preview shape
        let bottom_plate = BottomPlate::from_outline_and_insert_holders(
            cluster_outline.clone(),
            insert_holders.iter(),
            config.bottom_plate_thickness.into(),
        );
        let bottom_plate = Shape::new(&bottom_plate.into_tree(), bounds);

        let cluster_preview = combined_cluster.intersection(half_space);
        let preview = Shape::new(&cluster_preview, bounds);

        // Add the insert and interface PCB holders and switch cutouts
        let holders = Self::holders(
            insert_holders,
            interface_pcb.holder(bounds.diameter()),
            &cluster_outline,
        );
        let cluster = cluster
            .union(holders)
            .difference(Self::switch_cutouts(key_positions, shell_thickness));

        let case = cluster.difference(interface_pcb.cutouts(bounds.diameter()));
        let case = Shape::new(&case, bounds);

        Self {
            case,
            bottom_plate,
            preview,
            interface_pcb_position: interface_pcb.position,
        }
    }

    /// Calculates the switch cutouts from the given key positions.
    fn switch_cutouts(key_positions: &KeyPositions, shell_thickness: f64) -> Tree {
        const CUTOUT_SIZE: DVec2 = dvec2(14.0, 14.0);
        const PLATE_THICKNESS: f64 = 1.5;
        const PLATE_CLEARANCE: f64 = 1.0;

        let switch_cutout_size = CUTOUT_SIZE.extend(2.0 * (shell_thickness + EPSILON));
        let plate_cutout_height = shell_thickness - PLATE_THICKNESS;

        let switch_cutout = if plate_cutout_height > 0.0 {
            let plate_cutout_size = (CUTOUT_SIZE + dvec2(0.0, 2.0 * PLATE_CLEARANCE))
                .extend(plate_cutout_height + EPSILON);

            BoxShape::new(plate_cutout_size)
                .into_tree()
                .translate(vec_z(-plate_cutout_size.z / 2.0 - PLATE_THICKNESS))
                .union(BoxShape::new(switch_cutout_size))
        } else {
            BoxShape::new(switch_cutout_size).into_tree()
        };

        key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter())
            .chain(key_positions.thumb_keys.iter())
            .map(|&position| switch_cutout.affine(position))
            .reduce(|a, b| a.union(b))
            .expect("there is more than one key")
    }

    /// Calculates the holders from the insert and interface PCB holders
    /// an the cluster outline.
    fn holders(
        insert_holders: impl IntoIterator<Item = InsertHolder>,
        interface_pcb_holder: Tree,
        cluster_outline: &Tree,
    ) -> Tree {
        cluster_outline.intersection(
            insert_holders
                .into_iter()
                .map(IntoTree::into_tree)
                .reduce(|holders, holder| holders.union(holder))
                .expect("there is more than one insert holder for the finger cluster")
                .union(interface_pcb_holder),
        )
    }
}
