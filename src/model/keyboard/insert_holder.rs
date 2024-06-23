use fidget::context::Tree;
use glam::{dvec3, DVec2};

use crate::{
    config::Keyboard,
    model::{
        geometry::rotate_90_degrees,
        primitives::{Circle, Corner, Csg, Transforms},
    },
};

pub struct InsertHolder {
    point: DVec2,
    edge1: DVec2,
    edge2: DVec2,
}

impl InsertHolder {
    const INSERT_RADIUS: f64 = 2.0;
    const WALL_THICKNESS: f64 = 2.0;
    const HEIGHT: f64 = 7.0;
    const RADIUS: f64 = Self::INSERT_RADIUS + Self::WALL_THICKNESS;

    pub fn from_outline_points(vertices: &[DVec2], index: usize, config: &Keyboard) -> Self {
        let shell_thickness: f64 = config.shell_thickness.into();
        let circumference_distance: f64 = config.circumference_distance.into();

        let n = vertices.len();
        let previous_vertex = vertices[(index + n - 1) % n];
        let vertex = vertices[index];
        let next_vertex = vertices[(index + 1) % n];

        let edge1 = rotate_90_degrees(previous_vertex - vertex).normalize();
        let edge2 = rotate_90_degrees(vertex - next_vertex).normalize();

        let outwards_direction = (edge1 + edge2).normalize();
        let point = vertex
            + (circumference_distance
                - Self::INSERT_RADIUS
                - Self::WALL_THICKNESS.max(shell_thickness))
                * outwards_direction;

        Self {
            point,
            edge1,
            edge2,
        }
    }

}

impl From<InsertHolder> for Tree {
    fn from(insert_holder: InsertHolder) -> Self {
        let corner: Tree = Corner::new(insert_holder.edge1, insert_holder.edge2).into();
        let hole: Tree = Circle::new(InsertHolder::INSERT_RADIUS).into();
        let rounded_corner = corner.offset(InsertHolder::RADIUS).difference(hole);
        rounded_corner
            .translate(dvec3(insert_holder.point.x, insert_holder.point.y, 0.0))
            .extrude(0.0, InsertHolder::HEIGHT)
    }
}
