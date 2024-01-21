use glam::DVec3;
use opencascade::primitives::{IntoShape, JoinType, Shape};

use crate::model::{
    config::{self, PLATE_X_2},
    geometry::{zvec, Line, Plane},
    key_positions::ThumbKeys,
    util::{wire_from_points, MountSize},
};

const PLATE_Y_2: f64 = 1.5 * config::PLATE_Y_2;

pub struct ThumbCluster {
    pub shape: Shape,
}

impl ThumbCluster {
    pub fn new(thumb_keys: &ThumbKeys, circumference_distance: f64) -> Self {
        let size = MountSize::from_positions(thumb_keys.iter(), circumference_distance);

        let first_thumb_key = thumb_keys.first();
        let last_thumb_key = thumb_keys.last();

        let points = [
            first_thumb_key.translation
                - PLATE_X_2 * first_thumb_key.x_axis
                - PLATE_Y_2 * first_thumb_key.y_axis,
            first_thumb_key.translation - PLATE_X_2 * first_thumb_key.x_axis
                + PLATE_Y_2 * first_thumb_key.y_axis,
            last_thumb_key.translation
                + PLATE_X_2 * last_thumb_key.x_axis
                + PLATE_Y_2 * last_thumb_key.y_axis,
            last_thumb_key.translation + PLATE_X_2 * last_thumb_key.x_axis
                - PLATE_Y_2 * last_thumb_key.y_axis,
        ];

        let wire = wire_from_points(points, Plane::new(DVec3::ZERO, DVec3::Z));
        let face = wire.offset(circumference_distance, JoinType::Arc).to_face();
        let mount = face.extrude(zvec(size.height)).into_shape();

        let clearance = Self::clearance(thumb_keys, size);
        let shape = mount.subtract(&clearance).into();

        Self { shape }
    }

    fn clearance(thumb_keys: &ThumbKeys, mount_size: MountSize) -> Shape {
        let first = thumb_keys.first();
        let last = thumb_keys.last();

        let first_point = first.translation - PLATE_X_2 * first.x_axis;
        let last_point = last.translation + PLATE_X_2 * last.x_axis;
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

        let lower_plane = Plane::new(first.translation - PLATE_Y_2 * first.y_axis, first.y_axis);
        let upper_plane = Plane::new(first.translation + PLATE_Y_2 * first.y_axis, first.y_axis);

        let lower_face = wire_from_points(points.clone(), lower_plane).to_face();
        let upper_face = wire_from_points(points, upper_plane).to_face();

        lower_face
            .extrude(2.0 * PLATE_Y_2 * first.y_axis)
            .union(&lower_face.extrude(mount_size.length * DVec3::NEG_Y))
            .union(&upper_face.extrude(mount_size.length * DVec3::Y).into())
            .into()
    }
}
