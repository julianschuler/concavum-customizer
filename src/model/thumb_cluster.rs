use glam::{dvec2, dvec3, DVec3};
use opencascade::primitives::{IntoShape, JoinType, Shape, Wire};

use crate::model::{
    config::{self, PLATE_X_2},
    geometry::{zvec, Line, Plane, Project},
    key_positions::ThumbKeys,
    util::MountSize,
};

const PLATE_Y_2: f64 = 1.5 * config::PLATE_Y_2;

pub struct ThumbCluster {
    pub shape: Shape,
}

impl ThumbCluster {
    pub fn new(thumb_keys: &ThumbKeys, circumference_distance: f64) -> Self {
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

        let size = MountSize::from_points_and_positions(
            points.iter().map(|point| dvec2(point.x, point.y)),
            thumb_keys.iter(),
            circumference_distance,
        );

        let wire =
            Wire::from_ordered_points(points.iter().map(|point| dvec3(point.x, point.y, 0.0)))
                .expect("wire is created from more than 2 points");
        let wire = wire.offset(circumference_distance, JoinType::Arc);
        let mount = wire.to_face().extrude(zvec(size.height)).into_shape();

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
        let plane = Plane::new(first.translation, first.y_axis);

        // All points in the center, if any
        let points: Vec<_> = thumb_keys
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
            ])
            .map(|point| point.project_to(&plane))
            .collect();

        let lower_face =
            Wire::from_ordered_points(points.iter().map(|&point| point - PLATE_Y_2 * first.y_axis))
                .expect("wire is created from more than 2 points")
                .to_face();
        let upper_face =
            Wire::from_ordered_points(points.iter().map(|&point| point + PLATE_Y_2 * first.y_axis))
                .expect("wire is created from more than 2 points")
                .to_face();

        lower_face
            .extrude(2.0 * PLATE_Y_2 * first.y_axis)
            .union(&lower_face.extrude(mount_size.length * DVec3::NEG_Y))
            .union(&upper_face.extrude(mount_size.length * DVec3::Y).into())
            .into()
    }
}
