use glam::{dvec3, DAffine3, DVec2, DVec3};

use crate::{config::EPSILON, model::primitives::Bounds};

/// Bounded region containing a cluster.
pub struct ClusterBounds {
    min: DVec3,
    max: DVec3,
    pub size: DVec3,
}

impl ClusterBounds {
    /// Creates a cluster bound from key positions.
    pub fn from_positions<'a>(
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

    /// Combines two cluster bounds.
    pub fn union(&self, other: &Self) -> Self {
        let min = self.min.min(other.min);
        let max = self.max.max(other.max);
        let size = max - min;

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
