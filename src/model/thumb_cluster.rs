use fidget::context::Tree;
use glam::{dvec2, DVec2, DVec3, Vec3Swizzles};

use crate::{
    config::{Keyboard, PositiveDVec2, EPSILON, KEY_CLEARANCE},
    model::{
        geometry::{Line, Plane},
        insert_holder::InsertHolder,
        key_positions::ThumbKeys,
        primitives::{ConvexPolygon, Csg},
        util::{
            corner_point, prism_from_projected_points, sheared_prism_from_projected_points,
            side_point, ClusterBounds, Side, SideX, SideY,
        },
    },
};

pub struct ThumbCluster {
    pub cluster: Tree,
    pub key_clearance: Tree,
    pub insert_holder: Tree,
    pub bounds: ClusterBounds,
}

impl ThumbCluster {
    pub fn new(thumb_keys: &ThumbKeys, key_distance: &PositiveDVec2, config: &Keyboard) -> Self {
        let key_clearance = dvec2(
            key_distance.x + KEY_CLEARANCE,
            1.5 * key_distance.y + KEY_CLEARANCE,
        ) / 2.0;

        let bounds = ClusterBounds::from_positions(
            thumb_keys.iter(),
            &key_clearance,
            *config.circumference_distance,
        );

        let (outline, insert_holder) =
            Self::outline_and_insert_holder(thumb_keys, &key_clearance, config);
        let cluster_outline = outline.offset(*config.circumference_distance);

        let clearance = Self::clearance(thumb_keys, &key_clearance, &bounds);
        let cluster = cluster_outline.difference(clearance);

        let key_clearance = Self::key_clearance(thumb_keys, &key_clearance, &bounds);

        let insert_holder = cluster_outline.intersection(insert_holder);

        Self {
            cluster,
            key_clearance,
            insert_holder,
            bounds,
        }
    }

    fn outline_and_insert_holder(
        thumb_keys: &ThumbKeys,
        key_clearance: &DVec2,
        config: &Keyboard,
    ) -> (Tree, InsertHolder) {
        let first_thumb_key = thumb_keys.first();
        let last_thumb_key = thumb_keys.last();

        let points: Vec<_> = [
            corner_point(first_thumb_key, SideX::Left, SideY::Top, key_clearance),
            corner_point(first_thumb_key, SideX::Left, SideY::Bottom, key_clearance),
            corner_point(last_thumb_key, SideX::Right, SideY::Bottom, key_clearance),
            corner_point(last_thumb_key, SideX::Right, SideY::Top, key_clearance),
        ]
        .into_iter()
        .map(Vec3Swizzles::xy)
        .collect();

        let insert_holder = InsertHolder::from_vertices(&points, 1, config);

        (ConvexPolygon::new(points).into(), insert_holder)
    }

    fn clearance(thumb_keys: &ThumbKeys, key_clearance: &DVec2, bounds: &ClusterBounds) -> Tree {
        let width = bounds.size.x;
        let length = bounds.size.y;
        let height = bounds.size.z;

        let first = thumb_keys.first();
        let last = thumb_keys.last();

        let first_point = side_point(first, Side::Left, key_clearance);
        let last_point = side_point(last, Side::Right, key_clearance);
        let first_outwards_point = first_point + width * DVec3::NEG_X;
        let last_outwards_point = last_point + width * DVec3::X;
        let first_upwards_point = first_outwards_point + 2.0 * height * DVec3::Z;
        let last_upwards_point = last_outwards_point + 2.0 * height * DVec3::Z;

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

    fn key_clearance(
        thumb_keys: &ThumbKeys,
        key_clearance: &DVec2,
        bounds: &ClusterBounds,
    ) -> Tree {
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
                last_point + 2.0 * bounds.size.z * DVec3::Z,
                first_point + 2.0 * bounds.size.z * DVec3::Z,
                first_point,
            ]);

        let plane = Plane::new(side_point(first, Side::Bottom, key_clearance), first.y_axis);
        prism_from_projected_points(points, &plane, 2.0 * key_clearance.y)
    }
}
