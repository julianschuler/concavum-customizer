use glam::{dvec3, DAffine3, DMat2, DMat3};

use crate::model::{geometry::rotate_90_degrees, keyboard::InsertHolder};

pub struct InterfacePcb {
    pub position: DAffine3,
}

impl InterfacePcb {
    pub fn from_insert_holder(insert_holder: &InsertHolder) -> Self {
        const WIDTH: f64 = 36.0;

        let top_edge = insert_holder.outline_segment(WIDTH);
        let translation = dvec3(top_edge.start.x, top_edge.start.y, 0.0);

        let direction = (top_edge.end - top_edge.start).normalize();
        let rotation_matrix = DMat2::from_cols(direction, rotate_90_degrees(direction));

        let position =
            DAffine3::from_mat3_translation(DMat3::from_mat2(rotation_matrix), translation);

        Self { position }
    }
}
