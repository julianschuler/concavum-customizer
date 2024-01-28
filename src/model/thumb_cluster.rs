use glam::{dvec2, DVec2, DVec3};
use opencascade::primitives::{IntoShape, JoinType, Shape};

use crate::model::{
    config::{PositiveDVec2, KEY_CLEARANCE},
    geometry::{zvec, Line, Plane},
    key_positions::ThumbKeys,
    util::{corner_point, side_point, wire_from_points, MountSize, Side, SideX, SideY},
};

pub struct ThumbCluster {
    pub mount: Shape,
    pub key_clearance: Shape,
}

impl ThumbCluster {
    pub fn new(
        thumb_keys: &ThumbKeys,
        key_distance: &PositiveDVec2,
        circumference_distance: f64,
    ) -> Self {
        let key_clearance = dvec2(
            key_distance.x + KEY_CLEARANCE,
            1.5 * key_distance.y + KEY_CLEARANCE,
        ) / 2.0;

        let size =
            MountSize::from_positions(thumb_keys.iter(), &key_clearance, circumference_distance);

        let first_thumb_key = thumb_keys.first();
        let last_thumb_key = thumb_keys.last();

        let points = [
            corner_point(first_thumb_key, SideX::Left, SideY::Bottom, &key_clearance),
            corner_point(first_thumb_key, SideX::Left, SideY::Top, &key_clearance),
            corner_point(last_thumb_key, SideX::Right, SideY::Top, &key_clearance),
            corner_point(last_thumb_key, SideX::Right, SideY::Bottom, &key_clearance),
        ];

        let wire = wire_from_points(points, Plane::new(DVec3::ZERO, DVec3::Z));
        let face = wire.offset(circumference_distance, JoinType::Arc).to_face();
        let mount = face.extrude(zvec(size.height)).into_shape();

        let mount_clearance = Self::mount_clearance(thumb_keys, &key_clearance, &size);
        let mount = mount.subtract(&mount_clearance).into();

        let key_clearance = Self::key_clearance(thumb_keys, &key_clearance, &size);

        Self {
            mount,
            key_clearance,
        }
    }

    fn mount_clearance(
        thumb_keys: &ThumbKeys,
        key_clearance: &DVec2,
        mount_size: &MountSize,
    ) -> Shape {
        let first = thumb_keys.first();
        let last = thumb_keys.last();

        let first_point = side_point(first, Side::Left, key_clearance);
        let last_point = side_point(last, Side::Right, key_clearance);
        let first_outwards_point = first_point + mount_size.width * DVec3::NEG_X;
        let last_outwards_point = last_point + mount_size.width * DVec3::X;
        let first_upwards_point = first_outwards_point + 2.0 * mount_size.height * DVec3::Z;
        let last_upwards_point = last_outwards_point + 2.0 * mount_size.height * DVec3::Z;

        // All points in the center, if any
        let points = thumb_keys
            .windows(2)
            .filter_map(|window| {
                let position = window[0];
                let next_position = window[1];
                let line = Line::new(position.translation, position.x_axis);
                let plane = Plane::new(next_position.translation, next_position.z_axis);

                plane.intersection(&line)
            })
            .chain([
                last_point,
                last_outwards_point,
                last_upwards_point,
                first_upwards_point,
                first_outwards_point,
                first_point,
            ]);

        let lower_plane = Plane::new(side_point(first, Side::Bottom, key_clearance), first.y_axis);
        let upper_plane = Plane::new(side_point(first, Side::Top, key_clearance), first.y_axis);

        let lower_face = wire_from_points(points.clone(), lower_plane).to_face();
        let upper_face = wire_from_points(points, upper_plane).to_face();

        lower_face
            .extrude(2.0 * key_clearance.y * first.y_axis)
            .union(&lower_face.extrude(mount_size.length * DVec3::NEG_Y))
            .union(&upper_face.extrude(mount_size.length * DVec3::Y).into())
            .into()
    }

    fn key_clearance(
        thumb_keys: &ThumbKeys,
        key_clearance: &DVec2,
        mount_size: &MountSize,
    ) -> Shape {
        let first = thumb_keys.first();
        let last = thumb_keys.last();

        let first_point = side_point(first, Side::Left, key_clearance);
        let last_point = side_point(last, Side::Right, key_clearance);

        let points = thumb_keys
            .windows(2)
            .filter_map(|window| {
                let position = window[0];
                let next_position = window[1];
                let line = Line::new(position.translation, position.x_axis);
                let plane = Plane::new(next_position.translation, next_position.z_axis);

                plane.intersection(&line)
            })
            .chain([
                last_point,
                last_point + 2.0 * mount_size.height * DVec3::Z,
                first_point + 2.0 * mount_size.height * DVec3::Z,
                first_point,
            ]);

        let plane = Plane::new(side_point(first, Side::Bottom, key_clearance), first.y_axis);

        wire_from_points(points, plane)
            .to_face()
            .extrude(2.0 * key_clearance.y * first.y_axis)
            .into()
    }
}
