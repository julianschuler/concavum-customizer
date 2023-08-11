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
        Self { point, direction }
    }

    pub fn parametric_point(&self, parameter: f64) -> DVec3 {
        self.point + parameter * self.direction
    }

    pub fn intersection_parameter(&self, plane: &Plane) -> Option<f64> {
        let normal = plane.normal;
        let intersect_factor = normal.dot(plane.point - self.point) / self.direction.dot(normal);
        intersect_factor.is_finite().then_some(intersect_factor)
    }

    pub fn intersection(&self, plane: &Plane) -> Option<DVec3> {
        self.intersection_parameter(plane)
            .map(|factor| self.parametric_point(factor))
    }
}

pub struct Plane {
    point: DVec3,
    normal: DVec3,
}

#[allow(unused)]
impl Plane {
    pub fn new(point: DVec3, normal: DVec3) -> Self {
        Self { point, normal }
    }

    pub fn from_normal(normal: DVec3) -> Self {
        Self {
            point: DVec3::default(),
            normal,
        }
    }

    pub fn project(self, vector: DVec3) -> DVec3 {
        vector - vector.dot(self.normal) * self.normal
    }
}
