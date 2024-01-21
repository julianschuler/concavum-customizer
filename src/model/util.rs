use glam::{dvec3, DAffine3, DVec3};
use opencascade::primitives::{Solid, Wire};

use crate::model::{
    config::{PLATE_X_2, PLATE_Y_2},
    geometry::{Plane, Project},
};

/// Upper bound for the size of a mount
pub struct MountSize {
    pub width: f64,
    pub length: f64,
    pub height: f64,
}

impl MountSize {
    pub fn from_positions<'a>(
        positions: impl IntoIterator<Item = &'a DAffine3>,
        circumference_distance: f64,
    ) -> Self {
        const PADDING: f64 = PLATE_X_2 + PLATE_Y_2;

        let (min, max) = positions.into_iter().fold(
            (
                dvec3(f64::INFINITY, f64::INFINITY, f64::INFINITY),
                dvec3(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            ),
            |(min, max), point| (min.min(point.translation), max.max(point.translation)),
        );

        let width = max.x - min.x + 2.0 * circumference_distance + PADDING;
        let length = max.y - min.y + 2.0 * circumference_distance + PADDING;
        let height = max.z + PADDING;

        Self {
            width,
            length,
            height,
        }
    }
}

pub fn wire_from_points(points: impl IntoIterator<Item = DVec3>, plane: Plane) -> Wire {
    let points = points.into_iter().map(|point| point.project_to(&plane));
    Wire::from_ordered_points(points).expect("wire is created from more than 2 points")
}

pub fn project_points_to_plane_and_extrude(
    points: impl IntoIterator<Item = DVec3>,
    plane: Plane,
    height: f64,
) -> Solid {
    let direction = height * plane.normal();

    wire_from_points(points, plane).to_face().extrude(direction)
}
