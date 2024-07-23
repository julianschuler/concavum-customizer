use glam::{dvec2, dvec3, DMat3, DVec2, DVec3};

use crate::{
    geometry::Plane,
    primitives::{Bounds as ShapeBounds, EPSILON},
};

/// Bounded region containing a cluster given by two points.
#[derive(Clone)]
pub struct Bounds {
    /// Corner point with minimal coordinates.
    pub min: DVec3,
    /// Corner point with maximal coordinates.
    pub max: DVec3,
}

impl Bounds {
    /// Returns the size of the bounds
    pub fn size(&self) -> DVec3 {
        self.max - self.min
    }

    /// Combines two bounds.
    pub fn union(&self, other: &Self) -> Self {
        let min = self.min.min(other.min);
        let max = self.max.max(other.max);

        Self { min, max }
    }

    /// Mirrors the bound along the YZ-plane.
    pub fn mirror_yz(&self) -> Self {
        let min = dvec3(-self.max.x, self.min.y, self.min.z);
        let max = self.max;

        Self { min, max }
    }

    /// Returns the diameter of the bounds.
    pub fn diameter(&self) -> f64 {
        self.size().length()
    }

    /// Returns the unit vectors projected to a plane given by the normal.
    /// The vectors are scaled such that it translates every point inside
    /// the bound to the outside.
    pub fn projected_unit_vectors(&self, normal: DVec3) -> DMat3 {
        let plane = Plane::new(DVec3::ZERO, normal);

        let x_axis = plane.project_vector(DVec3::X).normalize_or_zero();
        let y_axis = plane.project_vector(DVec3::Y).normalize_or_zero();
        let z_axis = plane.project_vector(DVec3::Z).normalize_or_zero();

        self.diameter()
            * DMat3 {
                x_axis,
                y_axis,
                z_axis,
            }
    }

    /// Creates bounds from outline points and height.
    pub fn from_outline_points_and_height(
        outline_points: &[DVec2],
        height: f64,
        circumference_distance: f64,
    ) -> Self {
        let (min, max) = outline_points.iter().fold(
            (
                dvec2(f64::INFINITY, f64::INFINITY),
                dvec2(f64::NEG_INFINITY, f64::NEG_INFINITY),
            ),
            |(min, max), point| (point.min(min), point.max(max)),
        );

        let padding = dvec3(circumference_distance, circumference_distance, 0.0);
        let min = dvec3(min.x, min.y, 0.0) - padding;
        let max = dvec3(max.x, max.y, height) + padding;

        Self { min, max }
    }
}

impl From<Bounds> for ShapeBounds {
    fn from(bounds: Bounds) -> Self {
        let center = (bounds.min + bounds.max) / 2.0;
        let size = bounds.size().max_element() + 2.0 * EPSILON;

        Self::new(size, center)
    }
}
