use glam::{dvec3, DVec2, DVec3};

/// Returns a vector along the z-axis with the given z-value.
pub fn zvec(z: f64) -> DVec3 {
    dvec3(0.0, 0.0, z)
}

/// A line in 3D space.
pub struct Line {
    point: DVec3,
    direction: DVec3,
}

impl Line {
    /// Creates a line from a point on the line and a direction.
    pub fn new(point: DVec3, direction: DVec3) -> Self {
        let direction = direction.normalize();
        Self { point, direction }
    }

    /// Returns a parametric point on the line.
    pub fn parametric_point(&self, parameter: f64) -> DVec3 {
        self.point + parameter * self.direction
    }
}

/// A plane in 3D space.
pub struct Plane {
    point: DVec3,
    normal: DVec3,
}

impl Plane {
    /// Creates a line from a point on the plane and a normal vector.
    pub fn new(point: DVec3, normal: DVec3) -> Self {
        let normal = normal.normalize();
        Self { point, normal }
    }

    /// Calculates the signed distance to the plane.
    pub fn signed_distance_to(&self, point: DVec3) -> f64 {
        self.normal.dot(point - self.point)
    }

    /// Calculates the intersection between the plane and a line.
    ///
    /// Returns [`None`] if there is no intersection.
    pub fn intersection(&self, line: &Line) -> Option<DVec3> {
        let intersection_parameter =
            self.normal.dot(self.point - line.point) / line.direction.dot(self.normal);
        intersection_parameter
            .is_finite()
            .then(|| line.parametric_point(intersection_parameter))
    }

    /// Returns the normalized normal vector of the plane.
    pub fn normal(&self) -> DVec3 {
        self.normal
    }
}

/// A trait for projecting objects onto others.
pub trait Project<T> {
    /// Projects `self` onto the target.
    fn project_to(self, target: &T) -> Self;
}

impl Project<Plane> for DVec3 {
    fn project_to(self, plane: &Plane) -> Self {
        self - plane.signed_distance_to(self) * plane.normal
    }
}

/// Returns true if the triangle given by `p1`, `p2` and `p3` is counterclockwise or colinear.
pub fn counterclockwise_or_colinear(p1: DVec2, p2: DVec2, p3: DVec2) -> bool {
    (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x) <= 0.0
}
