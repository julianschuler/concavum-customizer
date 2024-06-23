use fidget::context::Tree;
use glam::{dvec3, DAffine3, DMat2, DMat3, DVec3};

use crate::model::{
    geometry::{rotate_90_degrees, zvec},
    keyboard::InsertHolder,
    primitives::{BoxShape, Csg, IntoTree, Transforms},
};

pub struct InterfacePcb {
    pub position: DAffine3,
}

impl InterfacePcb {
    const SIZE: DVec3 = dvec3(36.0, 42.0, 1.6);
    const HOLDER_SIZE: DVec3 = dvec3(3.0, 10.0, 1.0);

    pub fn from_insert_holder(insert_holder: &InsertHolder) -> Self {
        let top_edge = insert_holder.outline_segment(Self::SIZE.x);
        let translation = dvec3(top_edge.start.x, top_edge.start.y, Self::HOLDER_SIZE.z);

        let direction = (top_edge.end - top_edge.start).normalize();
        let rotation_matrix = DMat2::from_cols(direction, rotate_90_degrees(direction));

        let position =
            DAffine3::from_mat3_translation(DMat3::from_mat2(rotation_matrix), translation);

        Self { position }
    }

    pub fn holder(&self, bounds_diameter: f64) -> Tree {
        const HOLDER_WIDTH: f64 = 1.5;
        let height = Self::SIZE.z + Self::HOLDER_SIZE.z;

        let cutout = BoxShape::new(dvec3(Self::SIZE.x, bounds_diameter, bounds_diameter))
            .into_tree()
            .translate(zvec((bounds_diameter - height) / 2.0 + Self::HOLDER_SIZE.z))
            .union(BoxShape::new(dvec3(
                Self::SIZE.x - 2.0 * Self::HOLDER_SIZE.x,
                bounds_diameter,
                bounds_diameter,
            )));

        let translation = dvec3(
            Self::SIZE.x / 2.0,
            bounds_diameter / 2.0 - Self::HOLDER_SIZE.y,
            height / 2.0 - Self::HOLDER_SIZE.z,
        );

        BoxShape::new(dvec3(
            Self::SIZE.x + 2.0 * HOLDER_WIDTH,
            bounds_diameter,
            height,
        ))
        .into_tree()
        .difference(cutout)
        .affine(self.position * DAffine3::from_translation(translation))
    }
}
