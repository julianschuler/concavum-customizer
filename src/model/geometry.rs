use glam::{vec3a, Affine3A, Vec3A};

pub fn zvec(z: f32) -> Vec3A {
    vec3a(0.0, 0.0, z)
}

pub trait Translate {
    fn translate(self, translation: Vec3A) -> Self;
}

impl Translate for Affine3A {
    fn translate(self, translation: Vec3A) -> Self {
        Affine3A::from_translation(translation.into()) * self
    }
}

pub trait Rotate {
    fn rotate_x(self, angle: f32) -> Self;
    fn rotate_y(self, angle: f32) -> Self;
    fn rotate_z(self, angle: f32) -> Self;
}

impl Rotate for Affine3A {
    fn rotate_x(self, angle: f32) -> Self {
        Affine3A::from_rotation_x(angle) * self
    }

    fn rotate_y(self, angle: f32) -> Self {
        Affine3A::from_rotation_y(angle) * self
    }

    fn rotate_z(self, angle: f32) -> Self {
        Affine3A::from_rotation_z(angle) * self
    }
}

pub struct Line {
    point: Vec3A,
    direction: Vec3A,
}

impl Line {
    pub fn new(point: Vec3A, direction: Vec3A) -> Self {
        let direction = direction.normalize();
        Self { point, direction }
    }

    pub fn parametric_point(&self, parameter: f32) -> Vec3A {
        self.point + parameter * self.direction
    }
}

pub struct Plane {
    point: Vec3A,
    normal: Vec3A,
}

impl Plane {
    pub fn new(point: Vec3A, normal: Vec3A) -> Self {
        let normal = normal.normalize();
        Self { point, normal }
    }

    pub fn signed_distance_to(&self, point: Vec3A) -> f32 {
        self.normal.dot(point - self.point)
    }

    pub fn intersection(&self, line: &Line) -> Option<Vec3A> {
        let intersection_parameter =
            self.normal.dot(self.point - line.point) / line.direction.dot(self.normal);
        intersection_parameter
            .is_finite()
            .then(|| line.parametric_point(intersection_parameter))
    }

    pub fn normal(&self) -> Vec3A {
        self.normal
    }
}

pub trait Project<T> {
    fn project_to(self, target: &T) -> Self;
}

impl Project<Plane> for Vec3A {
    fn project_to(self, plane: &Plane) -> Self {
        self - plane.signed_distance_to(self) * plane.normal
    }
}
