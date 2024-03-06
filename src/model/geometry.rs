use glam::{dvec3, DVec2, DVec3};

pub fn zvec(z: f64) -> DVec3 {
    dvec3(0.0, 0.0, z)
}

pub struct Line {
    point: DVec3,
    direction: DVec3,
}

impl Line {
    pub fn new(point: DVec3, direction: DVec3) -> Self {
        let direction = direction.normalize();
        Self { point, direction }
    }

    pub fn parametric_point(&self, parameter: f64) -> DVec3 {
        self.point + parameter * self.direction
    }
}

pub struct Plane {
    point: DVec3,
    normal: DVec3,
}

impl Plane {
    pub fn new(point: DVec3, normal: DVec3) -> Self {
        let normal = normal.normalize();
        Self { point, normal }
    }

    pub fn signed_distance_to(&self, point: DVec3) -> f64 {
        self.normal.dot(point - self.point)
    }

    pub fn intersection(&self, line: &Line) -> Option<DVec3> {
        let intersection_parameter =
            self.normal.dot(self.point - line.point) / line.direction.dot(self.normal);
        intersection_parameter
            .is_finite()
            .then(|| line.parametric_point(intersection_parameter))
    }

    pub fn normal(&self) -> DVec3 {
        self.normal
    }
}

pub trait Project<T> {
    fn project_to(self, target: &T) -> Self;
}

impl Project<Plane> for DVec3 {
    fn project_to(self, plane: &Plane) -> Self {
        self - plane.signed_distance_to(self) * plane.normal
    }
}

/// Check whether a triangle given by three points p1, p2 and p2 is clockwise or colinear
pub fn counter_clockwise_or_colinear(p1: DVec2, p2: DVec2, p3: DVec2) -> bool {
    (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x) <= 0.0
}
