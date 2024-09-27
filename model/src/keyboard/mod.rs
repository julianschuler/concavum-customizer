mod bottom_plate;
mod bounds;
mod finger_cluster;
mod insert_holder;
mod thumb_cluster;

use std::iter::once;

use config::Config;
use fidget::context::Tree;
use glam::{dvec3, DAffine3, DVec3};

pub use bounds::Bounds;
pub use insert_holder::InsertHolder;

use crate::{
    geometry::Plane,
    key_positions::KeyPositions,
    matrix_pcb::MatrixPcb,
    primitives::{BoxShape, Csg, HalfSpace, IntoTree, RoundedCsg, Shape, Transforms},
    util::SideX,
};

use bottom_plate::BottomPlate;
use finger_cluster::FingerCluster;
use thumb_cluster::ThumbCluster;

/// A keyboard.
pub struct Keyboard {
    /// The left half of the keyboard.
    pub left_half: Shape,
    /// The right half of the keyboard.
    pub right_half: Shape,
    /// The bottom plate of the keyboard.
    pub bottom_plate: Shape,
    /// A simplified preview shape of the keyboard.
    pub preview: Shape,
    /// The position of the keys.
    pub key_positions: KeyPositions,
    /// The position of the interface PCB.
    pub interface_pcb_position: DAffine3,
    /// The matrix PCB.
    pub matrix_pcb: MatrixPcb,
}

impl Keyboard {
    /// Creates a keyboard from the given configuration.
    pub fn from_config(config: &Config) -> Self {
        let key_positions = KeyPositions::from_config(config);
        let matrix_pcb = MatrixPcb::from_positions(&key_positions);

        let finger_cluster = FingerCluster::new(&key_positions.columns, &config.keyboard);
        let thumb_cluster = ThumbCluster::new(&key_positions.thumb_keys, &config.keyboard);
        let bounds = finger_cluster.bounds.union(&thumb_cluster.bounds);
        let interface_pcb = finger_cluster.interface_pcb;
        let insert_holders: Vec<_> = finger_cluster
            .insert_holders
            .into_iter()
            .chain(once(thumb_cluster.insert_holder))
            .collect();
        let cluster_outline = finger_cluster.outline.union(thumb_cluster.outline);

        // Subtract key clearances from each other and combine the clusters
        let rounding_radius = config.keyboard.rounding_radius.into();
        let finger_key_clearance = finger_cluster.key_clearance;
        let finger_cluster = finger_cluster
            .cluster
            .rounded_difference(thumb_cluster.key_clearance, rounding_radius);
        let thumb_cluster = thumb_cluster
            .cluster
            .rounded_difference(finger_key_clearance, rounding_radius);
        let combined_cluster = finger_cluster.union(thumb_cluster);

        // Hollow out the combined cluster and cut off everthing below a Z value of 0
        let half_space = HalfSpace::new(Plane::new(DVec3::ZERO, DVec3::NEG_Z)).into_tree();
        let hollowed_cluster = combined_cluster.shell(config.keyboard.shell_thickness.into());
        let cluster = hollowed_cluster.intersection(half_space.clone());

        // Calculate the bottom plate and preview shape
        let bottom_plate = BottomPlate::from_outline_and_insert_holders(
            cluster_outline.clone(),
            insert_holders.iter(),
            config.keyboard.bottom_plate_thickness.into(),
        );
        let bottom_plate = Shape::new(&bottom_plate.into_tree(), bounds.into());

        let cluster_preview = combined_cluster.intersection(half_space);
        let preview = Shape::new(&cluster_preview, bounds.into());

        // Add the insert and interface PCB holders and switch cutouts
        let holders = Self::holders(
            insert_holders,
            interface_pcb.holder(bounds.diameter()),
            &cluster_outline,
        );
        let cluster = cluster
            .union(holders)
            .difference(Self::switch_cutouts(&key_positions));

        // Create the left and right halves by subtracting the interface PCB cutouts
        // and mirroring the left half along the YZ-plane
        let left_half = cluster
            .difference(interface_pcb.cutouts(SideX::Left, bounds.diameter()))
            .remap_xyz(Tree::x().neg(), Tree::y(), Tree::z());
        let right_half = cluster.difference(interface_pcb.cutouts(SideX::Right, bounds.diameter()));

        let left_half = Shape::new(&left_half, bounds.mirror_yz().into());
        let right_half = Shape::new(&right_half, bounds.into());

        Self {
            left_half,
            right_half,
            bottom_plate,
            preview,
            key_positions,
            interface_pcb_position: interface_pcb.position,
            matrix_pcb,
        }
    }

    /// Calculates the switch cutouts from the given key positions.
    fn switch_cutouts(key_positions: &KeyPositions) -> Tree {
        const CUTOUT_SIZE: DVec3 = dvec3(14.0, 14.0, 10.0);

        let switch_cutout = BoxShape::new(CUTOUT_SIZE).into_tree();

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
