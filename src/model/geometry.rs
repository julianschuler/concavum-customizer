use std::{cmp::Ordering, ops::Deref};

use glam::{dvec3, DAffine3, DVec2, DVec3};

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

    pub fn direction(&self) -> DVec3 {
        self.direction
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

pub struct BoundedPlane {
    plane: Plane,
    bounds: Vec<Plane>,
}

#[allow(unused)]
impl BoundedPlane {
    pub fn new(plane: Plane, bounds: impl IntoIterator<Item = Plane>) -> Self {
        let bounds = bounds.into_iter().collect();
        Self { plane, bounds }
    }

    pub fn intersection(&self, line: &Line) -> Option<DVec3> {
        self.plane.intersection(line).and_then(|point| {
            self.bounds
                .iter()
                .all(|bound| bound.signed_distance_to(point) >= 0.0)
                .then_some(point)
        })
    }
}

pub struct ConvexHull(Vec<DVec2>);

impl ConvexHull {
    pub fn from_points(points: Vec<DVec2>) -> Self {
        let mut sorted_points = points;
        sorted_points.sort_by(|a, b| {
            let cmp = a.x.total_cmp(&b.x);

            if cmp == Ordering::Equal {
                a.y.total_cmp(&b.y)
            } else {
                cmp
            }
        });

        let mut hull: Vec<DVec2> = Vec::new();

        // Lower hull
        for point in sorted_points.iter() {
            while hull.len() >= 2
                && clockwise_or_colinear(
                    hull.get(hull.len() - 2)
                        .expect("hull should have more than two elements"),
                    hull.last()
                        .expect("hull should have more than two elements"),
                    point,
                )
            {
                hull.pop();
            }
            hull.push(*point);
        }

        // Upper hull
        let lower_hull_size = hull.len();
        for point in sorted_points.iter().rev().skip(1) {
            while hull.len() > lower_hull_size
                && clockwise_or_colinear(
                    hull.get(hull.len() - 2)
                        .expect("hull should have more than two elements"),
                    hull.last()
                        .expect("hull should have more than two elements"),
                    point,
                )
            {
                hull.pop();
            }
            hull.push(*point);
        }

        // Last element is the same as the first one, remove it
        hull.pop();

        Self(hull)
    }
}

impl Deref for ConvexHull {
    type Target = Vec<DVec2>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Check whether a triangle given by three points p1, p2 and p2 is clockwise or colinear
fn clockwise_or_colinear(p1: &DVec2, p2: &DVec2, &p3: &DVec2) -> bool {
    (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x) <= 0.0
}
