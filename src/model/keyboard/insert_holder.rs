use fidget::context::Tree;
use glam::{dvec3, DVec2};

use crate::{
    config::Keyboard,
    model::{
        geometry::rotate_90_degrees,
        primitives::{Circle, Corner, Csg, Transforms},
    },
};

pub struct InsertHolder(Tree);

impl InsertHolder {
    pub fn from_outline_points(vertices: &[DVec2], index: usize, config: &Keyboard) -> Self {
        const INSERT_RADIUS: f64 = 2.0;
        const WALL_THICKNESS: f64 = 2.0;
        const HEIGHT: f64 = 7.0;

        let n = vertices.len();
        let previous_vertex = vertices[(index + n - 1) % n];
        let vertex = vertices[index];
        let next_vertex = vertices[(index + 1) % n];

        let edge1 = rotate_90_degrees(previous_vertex - vertex);
        let edge2 = rotate_90_degrees(vertex - next_vertex);

        let outwards_direction = (edge1.normalize() + edge2.normalize()).normalize();
        let point = vertex
            + (f64::from(config.circumference_distance)
                - INSERT_RADIUS
                - WALL_THICKNESS.max(config.shell_thickness.into()))
                * outwards_direction;

        let corner: Tree = Corner::new(edge1, edge2).into();
        let hole: Tree = Circle::new(INSERT_RADIUS).into();
        let rounded_corner = corner
            .offset(INSERT_RADIUS + WALL_THICKNESS)
            .difference(hole);
        let tree = rounded_corner
            .translate(dvec3(point.x, point.y, 0.0))
            .extrude(0.0, HEIGHT);

        Self(tree)
    }
}

impl From<InsertHolder> for Tree {
    fn from(insert_holder: InsertHolder) -> Self {
        insert_holder.0
    }
}
