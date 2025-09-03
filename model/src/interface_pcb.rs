use std::f64::consts::{FRAC_PI_2, PI};

use fidget::context::Tree;
use glam::{dvec2, dvec3, DAffine3, DMat2, DMat3, DVec2, DVec3, Vec3Swizzles};

use crate::{
    geometry::{rotate_90_degrees, vec_y, vec_z, Tangent},
    keyboard::InsertHolder,
    primitives::{BoxShape, Circle, Csg, IntoTree, Rectangle, Transforms},
};

/// A PCB containing the interfaces to the outside (USB and TRRS) and inside (FFC).
pub struct InterfacePcb {
    /// The position of the interface PCB.
    pub position: DAffine3,
}

impl InterfacePcb {
    const SIZE: DVec3 = dvec3(36.0, 42.0, 1.6);
    const HOLDER_THICKNESS: f64 = 1.0;
    const TOLERANCE: f64 = 0.1;

    /// Creates a new interface PCB from the given insert holder and outline points.
    pub fn new(insert_holder: &InsertHolder, points: &[DVec2], outline_offset: f64) -> Self {
        let tangent = insert_holder.tangent();
        let offset =
            Self::offset_along_tangent(tangent, &points[..=insert_holder.index()], outline_offset);

        let Tangent {
            point,
            direction,
            normal,
        } = tangent;

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
        const RETENTION_CLIP_WIDTH: f64 = 7.0;
        const RETENTION_CLIP_DEPTH: f64 = 0.5;

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
            (Self::SIZE.xy() - dvec2(2.0 * WIDTH, WIDTH - Self::TOLERANCE)).extend(bounds_diameter);

        let holder = BoxShape::new(holder_size).into_tree().translate(dvec3(
            0.0,
            bounds_diameter / 2.0 + y_offset,
            z_offset,
        ));
        let retention_clip = Circle::new(WIDTH / 2.0 + RETENTION_CLIP_DEPTH)
            .into_tree()
            .extrude(
                Self::SIZE.x / 2.0 - RETENTION_CLIP_WIDTH,
                Self::SIZE.x / 2.0 + WIDTH + Self::TOLERANCE,
            )
            .affine(DAffine3 {
                matrix3: DMat3::from_rotation_y(FRAC_PI_2),
                translation: dvec3(
                    0.0,
                    y_offset + WIDTH / 2.0,
                    Self::SIZE.z + (RETENTION_CLIP_DEPTH * (WIDTH + RETENTION_CLIP_DEPTH)).sqrt(),
                ),
            })
            .remap_xyz(Tree::x().abs(), Tree::y(), Tree::z());
        let pcb_cutout = BoxShape::new(pcb_cutout_size)
            .into_tree()
            .translate(vec_z(bounds_diameter / 2.0));
        let bottom_cutout = BoxShape::new(bottom_cutout_size)
            .into_tree()
            .translate(vec_y(f64::midpoint(WIDTH, Self::TOLERANCE)));
        let translation = dvec3(Self::SIZE.x / 2.0, -Self::SIZE.y / 2.0, 0.0);

        holder
            .difference(pcb_cutout.union(bottom_cutout))
            .union(retention_clip)
            .affine(self.position * DAffine3::from_translation(translation))
    }

    /// Returns the cutouts required for the USB and TRRS ports for the given side.
    pub fn cutouts(&self, bounds_diameter: f64) -> Tree {
        const USB_SIZE: DVec2 = dvec2(9.0, 3.2);
        const USB_RADIUS: f64 = 1.1;
        const USB_OFFSET: DVec3 = dvec3(24.9, 0.0, 3.2);
        const JACK_RADIUS: f64 = 2.5;
        const JACK_OFFSET: DVec3 = dvec3(5.4, 0.0, 2.45);

        let jack_cutout = Circle::new(JACK_RADIUS + Self::TOLERANCE)
            .into_tree()
            .extrude(-Self::SIZE.y / 2.0, bounds_diameter);

        let rotation_x = DAffine3::from_rotation_x(-PI / 2.0);
        let height_offset = vec_z(Self::SIZE.z);

        let usb_cutout = Rectangle::new(USB_SIZE - DVec2::splat(2.0 * USB_RADIUS))
            .into_tree()
            .offset(USB_RADIUS + Self::TOLERANCE)
            .extrude(-Self::SIZE.y / 2.0, bounds_diameter);

        let usb_translation = DAffine3::from_translation(USB_OFFSET + height_offset);
        let jack_translation = DAffine3::from_translation(JACK_OFFSET + height_offset);

        usb_cutout
            .affine(self.position * usb_translation * rotation_x)
            .union(jack_cutout.affine(self.position * jack_translation * rotation_x))
    }

    /// Calculates the offset of the interface PCB along the tangent direction to the tangent point.
    fn offset_along_tangent(tangent: Tangent, points: &[DVec2], outline_offset: f64) -> f64 {
        let Tangent {
            point,
            direction,
            normal,
        } = tangent;
        let mut offset = f64::INFINITY;
        let width = Self::SIZE.x + 2.0 * Self::TOLERANCE;

        for window in points.windows(2).rev() {
            let left_point = window[1];
            let right_point = window[0];

            let edge_direction = (right_point - left_point).normalize();
            let edge_normal = rotate_90_degrees(edge_direction);

            let left_difference = left_point + outline_offset * edge_normal - point;
            let right_difference = right_point + outline_offset * edge_normal - point;

            let left_distance = left_difference.dot(normal);
            let right_distance = right_difference.dot(normal);
            let left_offset = left_difference.dot(direction);
            let right_offset = right_difference.dot(direction);

            if left_distance >= width {
                return offset.min(left_offset);
            } else if right_distance >= width {
                let projected_difference = right_difference
                    - edge_direction * (right_distance - width) / edge_direction.dot(normal);

                return offset.min(projected_difference.dot(direction));
            }

            offset = offset.min(left_offset.min(right_offset));
        }

        offset
    }
}
