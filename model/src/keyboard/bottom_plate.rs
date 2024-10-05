use fidget::context::Tree;
use glam::DVec2;

use crate::{
    key_positions,
    keyboard::InsertHolder,
    primitives::{Circle, Csg, IntoTree, Transforms},
};

/// A bottom plate of the keyboard.
pub struct BottomPlate {
    outline: Tree,
    hole_positions: Vec<DVec2>,
    thickness: f64,
}

impl BottomPlate {
    /// Creates a bottom plate with the given thickness from an outline and insert holders.
    pub fn from_outline_and_insert_holders<'a>(
        outline: Tree,
        insert_holders: impl IntoIterator<Item = &'a InsertHolder>,
        thickness: f64,
    ) -> Self {
        let holes = insert_holders
            .into_iter()
            .map(InsertHolder::center)
            .collect();

        Self {
            outline,
            hole_positions: holes,
            thickness,
        }
    }

    /// Returns the DXF output of the bottom plate.
    pub fn dxf<'a>(
        finger_cluster_outline_points: &[DVec2],
        thumb_cluster_outline_points: &[DVec2],
        insert_holders: impl IntoIterator<Item = &'a InsertHolder>,
        circumference_distance: f64,
    ) -> String {
        String::new()
    }
}

impl From<BottomPlate> for Tree {
    fn from(bottom_plate: BottomPlate) -> Self {
        const HOLE_RADIUS: f64 = 1.6;

        bottom_plate
            .hole_positions
            .into_iter()
            .map(|hole_position| {
                let translation = hole_position.extend(0.0);

                Circle::new(HOLE_RADIUS).into_tree().translate(translation)
            })
            .fold(bottom_plate.outline, |outline, hole| {
                outline.difference(hole)
            })
            .extrude(-bottom_plate.thickness, 0.0)
    }
}
