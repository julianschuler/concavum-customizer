use config::Keyboard;
use fidget::context::Tree;
use glam::DVec3;

use crate::{
    geometry::{Line, Plane},
    key_positions::ThumbKeys,
    keyboard::InsertHolder,
    primitives::{Bounds, ConvexPolygon, Csg, IntoTree, RoundedCsg, EPSILON},
    util::{
        bounds_from_outline_points_and_height, prism_from_projected_points, projected_unit_vectors,
        sheared_prism_from_projected_points, side_point, Side,
    },
};

/// A thumb cluster containing the thumb keys.
pub struct ThumbCluster {
    /// The tree of the thumb cluster.
    pub cluster: Tree,
    /// The outline of the thumb cluster.
    pub outline: Tree,
    /// The clearance required for the thumb keys.
    pub key_clearance: Tree,
    /// The insert holder positioned within the thumb cluster.
    pub insert_holder: InsertHolder,
    /// The bounds of the thumb cluster.
    pub bounds: Bounds,
}

impl ThumbCluster {
    /// Creates a new thumb cluster from the given thumb keys and configuration.
    pub fn new(thumb_keys: &ThumbKeys, config: &Keyboard) -> Self {
        let outline_points = thumb_keys.outline_points();
        let cluster_height = thumb_keys.max_z() + thumb_keys.key_clearance.length();
        let circumference_distance = config.circumference_distance.into();
        let outline_offset = circumference_distance - f64::from(config.shell_thickness);

        let bounds = bounds_from_outline_points_and_height(
            &outline_points,
            cluster_height,
            circumference_distance,
        );
        let insert_holder = InsertHolder::from_outline_points(&outline_points, 1, outline_offset);

        let cluster_outline = ConvexPolygon::new(outline_points)
            .into_tree()
            .offset(circumference_distance);
        let cluster = cluster_outline.extrude(-cluster_height, cluster_height);

        let clearance = Self::clearance(thumb_keys, bounds);
        let cluster = cluster.rounded_difference(clearance, config.rounding_radius.into());

        let key_clearance = Self::key_clearance(thumb_keys, bounds);

        Self {
            cluster,
            outline: cluster_outline,
            key_clearance,
            insert_holder,
            bounds,
        }
    }

    fn clearance(thumb_keys: &ThumbKeys, bounds: Bounds) -> Tree {
        let key_clearance = thumb_keys.key_clearance;

        let first = thumb_keys.first();
        let last = thumb_keys.last();
        let length = bounds.size().y;
        let bounds = projected_unit_vectors(first.y_axis, bounds);

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

    fn key_clearance(thumb_keys: &ThumbKeys, bounds: Bounds) -> Tree {
        let key_clearance = thumb_keys.key_clearance;
        let first = thumb_keys.first();
        let last = thumb_keys.last();

        let first_point = side_point(first, Side::Left, key_clearance);
        let last_point = side_point(last, Side::Right, key_clearance);
        let bounds = projected_unit_vectors(first.y_axis, bounds);

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
