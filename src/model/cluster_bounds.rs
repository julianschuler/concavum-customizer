use glam::{dvec3, DAffine3, DMat3, DVec2, DVec3};

use crate::{
    config::EPSILON,
    model::{
        geometry::Plane,
        key_positions::{Columns, ThumbKeys},
        primitives::Bounds,
    },
};

/// Bounded region containing a cluster.
pub struct ClusterBounds {
    min: DVec3,
    max: DVec3,
    size: DVec3,
}

impl ClusterBounds {
    /// Creates a cluster bound from columns.
    pub fn from_columns(columns: &Columns, circumference_distance: f64) -> Self {
        Self::from_positions(
            columns.iter().flat_map(|column| column.iter()),
            &columns.key_clearance,
            circumference_distance,
        )
    }

    /// Creates a cluster bound from thumb keys.
    pub fn from_thumb_keys(thumb_keys: &ThumbKeys, circumference_distance: f64) -> Self {
        Self::from_positions(
            thumb_keys.iter(),
            &thumb_keys.key_clearance,
            circumference_distance,
        )
    }

    /// Combines two cluster bounds.
    pub fn union(&self, other: &Self) -> Self {
        let min = self.min.min(other.min);
        let max = self.max.max(other.max);
        let size = max - min;

        Self { min, max, size }
    }

    /// Returns the diameter of the bounds.
    pub fn diameter(&self) -> f64 {
        self.size.length()
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

    /// Creates a cluster bound from key positions and clearances.
    fn from_positions<'a>(
        positions: impl IntoIterator<Item = &'a DAffine3>,
        key_clearance: &DVec2,
        circumference_distance: f64,
    ) -> Self {
        let (min, max) = positions.into_iter().fold(
            (
                dvec3(f64::INFINITY, f64::INFINITY, f64::INFINITY),
                dvec3(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            ),
            |(min, max), point| (min.min(point.translation), max.max(point.translation)),
        );

        let padding = key_clearance.length() + circumference_distance;
        let padding = dvec3(padding, padding, padding);
        let max = max + padding;
        let min = min - padding;
        let size = max - dvec3(min.x, min.y, 0.0);

        Self { min, max, size }
    }
}

impl From<ClusterBounds> for Bounds {
    fn from(bounds: ClusterBounds) -> Self {
        let center = (bounds.min + bounds.max) / 2.0;
        let size = bounds.size.max_element() + 2.0 * EPSILON;

        Self::new(size, center)
    }
}
