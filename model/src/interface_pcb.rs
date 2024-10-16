use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_4, PI, SQRT_2};

use fidget::context::Tree;
use glam::{dvec2, dvec3, DAffine3, DMat2, DMat3, DVec2, DVec3, Vec3Swizzles};

use crate::{
    geometry::{rotate_90_degrees, vec_y, vec_z, Tangent},
    keyboard::InsertHolder,
    primitives::{BoxShape, Circle, Csg, IntoTree, Rectangle, Transforms},
    util::SideX,
};

/// A PCB containing the interfaces to the outside (USB and TRRS) and inside (FPC).
pub struct InterfacePcb {
    /// The position of the interface PCB.
    pub position: DAffine3,
}

impl InterfacePcb {
    const SIZE: DVec3 = dvec3(36.0, 42.0, 1.6);
    const HOLDER_THICKNESS: f64 = 1.0;
    const TOLERANCE: f64 = 0.1;

    /// Creates a new interface PCB from the given insert holder and outline points.
    pub fn new(insert_holder: &InsertHolder, vertices: &[DVec2], offset: f64) -> Self {
        let width = Self::SIZE.x + 2.0 * Self::TOLERANCE;

        let Tangent { point, direction } = insert_holder.tangent();
        let normal = -rotate_90_degrees(direction);

        let mut predicate = true;
        let offset = vertices[..insert_holder.index()]
            .iter()
            .rev()
            .map(|vertex| vertex - point)
            .take_while(|difference| {
                let previous_predicate = predicate;
                predicate = difference.dot(normal) <= width;
                previous_predicate
            })
            .map(|difference| difference.dot(direction))
            .min_by(f64::total_cmp)
            .expect("the first value is always yielded by take_while")
            + offset;

        let rotation_matrix = DMat2::from_cols(normal, direction);
        let translation =
            (point + Self::TOLERANCE * normal + (offset - Self::TOLERANCE) * direction)
                .extend(Self::HOLDER_THICKNESS);
        let position =
            DAffine3::from_mat3_translation(DMat3::from_mat2(rotation_matrix), translation);

        Self { position }
    }

    /// Returns the holder for the interface PCB.
    pub fn holder(&self, bounds_diameter: f64) -> Tree {
        const WIDTH: f64 = 1.5;
        const RETENTION_CLIP_SIZE: DVec3 = dvec3(7.0, FRAC_1_SQRT_2 * WIDTH, SQRT_2 * WIDTH);

        let y_offset = -Self::SIZE.y / 2.0 - WIDTH - Self::TOLERANCE;
        let z_offset = (Self::SIZE.z - Self::HOLDER_THICKNESS) / 2.0;

        let holder_size = dvec3(
            Self::SIZE.x + 2.0 * (WIDTH + Self::TOLERANCE),
            bounds_diameter,
            Self::SIZE.z + Self::HOLDER_THICKNESS,
        );
        let pcb_cutout_size =
            (Self::SIZE.xy() + DVec2::splat(2.0 * Self::TOLERANCE)).extend(bounds_diameter);
        let bottom_cutout_size =
            (Self::SIZE.xy() - dvec2(2.0 * WIDTH, WIDTH)).extend(bounds_diameter);

        let holder = BoxShape::new(holder_size).into_tree().translate(dvec3(
            0.0,
            bounds_diameter / 2.0 + y_offset,
            z_offset,
        ));
        let retention_clip = BoxShape::new(RETENTION_CLIP_SIZE)
            .into_tree()
            .affine(
                DAffine3::from_translation(dvec3(
                    (Self::SIZE.x - RETENTION_CLIP_SIZE.x) / 2.0,
                    y_offset + WIDTH / 2.0,
                    Self::SIZE.z,
                )) * DAffine3::from_rotation_x(-FRAC_PI_4)
                    * DAffine3::from_translation(vec_z(RETENTION_CLIP_SIZE.y / 2.0)),
            )
            .remap_xyz(Tree::x().abs(), Tree::y(), Tree::z());
        let pcb_cutout = BoxShape::new(pcb_cutout_size)
            .into_tree()
            .translate(vec_z(bounds_diameter / 2.0));
        let bottom_cutout = BoxShape::new(bottom_cutout_size)
            .into_tree()
            .translate(vec_y(WIDTH / 2.0));
        let translation = dvec3(Self::SIZE.x / 2.0, -Self::SIZE.y / 2.0, 0.0);

        holder
            .difference(pcb_cutout.union(bottom_cutout))
            .union(retention_clip)
            .affine(self.position * DAffine3::from_translation(translation))
    }

    /// Returns the cutouts required for the USB and TRRS ports for the given side.
    pub fn cutouts(&self, side: SideX, bounds_diameter: f64) -> Tree {
        const USB_SIZE: DVec2 = dvec2(9.0, 3.2);
        const USB_RADIUS: f64 = 1.1;
        const USB_OFFSET: DVec3 = dvec3(24.9, 0.0, 3.2);
        const JACK_RADIUS: f64 = 3.0;
        const JACK_OFFSET_LEFT: DVec3 = dvec3(5.4, 0.0, 2.45);
        const JACK_OFFSET_RIGHT: DVec3 = dvec3(10.7, 0.0, 2.45);

        let jack_cutout = Circle::new(JACK_RADIUS + Self::TOLERANCE)
            .into_tree()
            .extrude(-Self::SIZE.y / 2.0, bounds_diameter);

        let rotation_x = DAffine3::from_rotation_x(-PI / 2.0);
        let height_offset = vec_z(Self::SIZE.z);

        if matches!(side, SideX::Left) {
            let usb_cutout = Rectangle::new(USB_SIZE - DVec2::splat(2.0 * USB_RADIUS))
                .into_tree()
                .offset(USB_RADIUS + Self::TOLERANCE)
                .extrude(-Self::SIZE.y / 2.0, bounds_diameter);

            let usb_translation = DAffine3::from_translation(USB_OFFSET + height_offset);
            let jack_translation = DAffine3::from_translation(JACK_OFFSET_LEFT + height_offset);

            usb_cutout
                .affine(self.position * usb_translation * rotation_x)
                .union(jack_cutout.affine(self.position * jack_translation * rotation_x))
        } else {
            let jack_translation = DAffine3::from_translation(JACK_OFFSET_RIGHT + height_offset);

            jack_cutout.affine(self.position * jack_translation * rotation_x)
        }
    }
}
