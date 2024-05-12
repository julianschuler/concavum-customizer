use fidget::context::Tree;
use glam::DVec3;

use crate::{
    config::{Keyboard, EPSILON},
    model::{
        geometry::{Line, Plane},
        key_positions::ThumbKeys,
        keyboard::{Bounds, InsertHolder},
        primitives::{ConvexPolygon, Csg, RoundedCsg},
        util::{
            prism_from_projected_points, sheared_prism_from_projected_points, side_point, Side,
        },
    },
};

pub struct ThumbCluster {
    pub cluster: Tree,
    pub key_clearance: Tree,
    pub insert_holder: Tree,
    pub bounds: Bounds,
}

impl ThumbCluster {
    pub fn new(thumb_keys: &ThumbKeys, config: &Keyboard) -> Self {
        let outline_points = thumb_keys.outline_points();
        let cluster_height = thumb_keys.max_z() + thumb_keys.key_clearance.length();

        let bounds = Bounds::from_outline_points_and_height(
            &outline_points,
            cluster_height,
            *config.circumference_distance,
        );
        let insert_holder: Tree =
            InsertHolder::from_outline_points(&outline_points, 1, config).into();

        let outline: Tree = ConvexPolygon::new(outline_points).into();
        let cluster_outline = outline.offset(*config.circumference_distance);
        let cluster = cluster_outline.extrude(-cluster_height, cluster_height);

        let clearance = Self::clearance(thumb_keys, &bounds);
        let cluster = cluster.rounded_difference(clearance, config.rounding_radius);

        let key_clearance = Self::key_clearance(thumb_keys, &bounds);
        let insert_holder = insert_holder.intersection(cluster_outline);

        Self {
            cluster,
            key_clearance,
            insert_holder,
            bounds,
        }
    }

    fn clearance(thumb_keys: &ThumbKeys, bounds: &Bounds) -> Tree {
        let key_clearance = &thumb_keys.key_clearance;

        let first = thumb_keys.first();
        let last = thumb_keys.last();
        let length = bounds.size.y;
        let bounds = bounds.projected_unit_vectors(first.y_axis);

        let first_point = side_point(first, Side::Left, key_clearance);
        let last_point = side_point(last, Side::Right, key_clearance);
        let first_outwards_point = first_point - bounds.x_axis;
        let last_outwards_point = last_point + bounds.x_axis;
        let first_upwards_point = first_outwards_point + bounds.z_axis;
        let last_upwards_point = last_outwards_point + bounds.z_axis;

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
            .collect();

        let lower_point = side_point(first, Side::Bottom, key_clearance);
        let lower_plane = Plane::new(lower_point, -first.y_axis);
        let middle_plane = Plane::new(lower_point - EPSILON * first.y_axis, first.y_axis);
        let upper_plane = Plane::new(side_point(first, Side::Top, key_clearance), first.y_axis);

        let lower = sheared_prism_from_projected_points(
            points.iter().copied(),
            &lower_plane,
            length,
            DVec3::Y,
        );
        let middle = prism_from_projected_points(
            points.iter().copied(),
            &middle_plane,
            2.0 * (key_clearance.y + EPSILON),
        );
        let upper = sheared_prism_from_projected_points(points, &upper_plane, length, DVec3::Y);

        let union = lower.union(middle);
        union.union(upper)
    }

    fn key_clearance(thumb_keys: &ThumbKeys, bounds: &Bounds) -> Tree {
        let key_clearance = &thumb_keys.key_clearance;
        let first = thumb_keys.first();
        let last = thumb_keys.last();

        let first_point = side_point(first, Side::Left, key_clearance);
        let last_point = side_point(last, Side::Right, key_clearance);
        let bounds = bounds.projected_unit_vectors(first.y_axis);

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
                last_point + bounds.z_axis,
                first_point + bounds.z_axis,
                first_point,
            ]);

        let plane = Plane::new(side_point(first, Side::Bottom, key_clearance), first.y_axis);
        prism_from_projected_points(points, &plane, 2.0 * key_clearance.y)
    }
}
