use std::iter::{Zip, Skip};

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
