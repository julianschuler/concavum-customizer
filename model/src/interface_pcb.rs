use std::f64::consts::PI;

use fidget::context::Tree;
use glam::{dvec2, dvec3, DAffine3, DMat2, DMat3, DVec2, DVec3};

use crate::{
    geometry::{rotate_90_degrees, zvec},
    keyboard::InsertHolder,
    primitives::{BoxShape, Circle, Csg, IntoTree, Rectangle, Transforms},
};

/// A PCB containing the interfaces to the outside (USB and TRRS) and inside (FPC).
pub struct InterfacePcb {
    /// The position of the interface PCB.
    pub position: DAffine3,
}

impl InterfacePcb {
    const SIZE: DVec3 = dvec3(36.0, 42.0, 1.6);
    const HOLDER_THICKNESS: f64 = 1.0;

    /// Creates a new interface PCB using the given insert holder.
    pub fn from_insert_holder(insert_holder: &InsertHolder) -> Self {
        let top_edge = insert_holder.outline_segment(Self::SIZE.x);
        let translation = dvec3(top_edge.start.x, top_edge.start.y, Self::HOLDER_THICKNESS);

        let direction = (top_edge.end - top_edge.start).normalize();
        let rotation_matrix = DMat2::from_cols(direction, rotate_90_degrees(direction));

        let position =
            DAffine3::from_mat3_translation(DMat3::from_mat2(rotation_matrix), translation);

        Self { position }
    }

    /// Returns the holder for the interface PCB.
    pub fn holder(&self, bounds_diameter: f64) -> Tree {
        const WIDTH: f64 = 1.5;
        const LENGTH: f64 = 10.0;
        const BRACKET_WIDTH: f64 = 5.0;
        let height = Self::SIZE.z + Self::HOLDER_THICKNESS;

        let cutout =
            BoxShape::new(dvec3(Self::SIZE.x, bounds_diameter, bounds_diameter)).into_tree();
        let cutout = cutout
            .translate(zvec(
                (bounds_diameter - height) / 2.0 + Self::HOLDER_THICKNESS,
            ))
            .union(cutout.translate(dvec3(0.0, BRACKET_WIDTH, 0.0)));

        let translation = dvec3(
            Self::SIZE.x / 2.0,
            bounds_diameter / 2.0 - LENGTH,
            height / 2.0 - Self::HOLDER_THICKNESS,
        );

        BoxShape::new(dvec3(Self::SIZE.x + 2.0 * WIDTH, bounds_diameter, height))
            .into_tree()
            .difference(cutout)
            .affine(self.position * DAffine3::from_translation(translation))
    }

    /// Returns the cutouts required for the USB and TRRS ports.
    pub fn cutouts(&self, bounds_diameter: f64) -> Tree {
        const USB_SIZE: DVec2 = dvec2(9.2, 3.4);
        const USB_RADIUS: f64 = 1.1;
        const USB_OFFSET: DVec3 = dvec3(24.9, 0.0, 3.2);
        const JACK_RADIUS: f64 = 3.1;
        const JACK_OFFSET_LEFT: DVec3 = dvec3(5.4, 0.0, 2.45);
        const JACK_OFFSET_RIGHT: DVec3 = dvec3(10.7, 0.0, 2.45);

        let jack_cutout = Circle::new(JACK_RADIUS)
            .into_tree()
            .extrude(-Self::SIZE.y, bounds_diameter);
        let usb_cutout = Rectangle::new(USB_SIZE - dvec2(USB_RADIUS, USB_RADIUS))
            .into_tree()
            .offset(USB_RADIUS)
            .extrude(-Self::SIZE.y, bounds_diameter);

        let rotation_x = DAffine3::from_rotation_x(-PI / 2.0);
        let height_offset = zvec(Self::SIZE.z);

        let jack_translation_left = DAffine3::from_translation(JACK_OFFSET_LEFT + height_offset);
        let jack_translation_right = DAffine3::from_translation(JACK_OFFSET_RIGHT + height_offset);
        let usb_translation = DAffine3::from_translation(USB_OFFSET + height_offset);

        usb_cutout
            .affine(self.position * usb_translation * rotation_x)
            .union(jack_cutout.affine(self.position * jack_translation_left * rotation_x))
            .remap_xyz(Tree::x().neg(), Tree::y(), Tree::z())
            .union(jack_cutout.affine(self.position * jack_translation_right * rotation_x))
    }
}
