use std::iter::{Skip, Zip};

use glam::{dvec3, DAffine3, DVec3};

pub fn zvec(z: f64) -> DVec3 {
    dvec3(0.0, 0.0, z)
}

pub trait Translate {
    fn translate(self, translation: DVec3) -> Self;
}

impl Translate for DAffine3 {
    fn translate(self, translation: DVec3) -> Self {
        DAffine3::from_translation(translation) * self
    }
}

pub trait Rotate {
    fn rotate_x(self, angle: f64) -> Self;
    fn rotate_y(self, angle: f64) -> Self;
    fn rotate_z(self, angle: f64) -> Self;
}

impl Rotate for DAffine3 {
    fn rotate_x(self, angle: f64) -> Self {
        DAffine3::from_rotation_x(angle) * self
    }

    fn rotate_y(self, angle: f64) -> Self {
        DAffine3::from_rotation_y(angle) * self
    }

    fn rotate_z(self, angle: f64) -> Self {
        DAffine3::from_rotation_z(angle) * self
    }
}

pub trait ZipNeighbors<T> {
    fn zip_neighbors(self) -> Zip<T, Skip<T>>;
}

impl<T: Iterator + Clone> ZipNeighbors<T> for T {
    fn zip_neighbors(self) -> Zip<T, Skip<T>> {
        let shifted_iterator = self.clone().skip(1);
        self.zip(shifted_iterator)
    }
}

pub struct Line {
    point: DVec3,
    direction: DVec3,
}

#[allow(unused)]
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
}
