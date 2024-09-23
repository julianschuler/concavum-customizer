use glam::{dvec2, dvec3, DMat2, DMat3, DVec2, DVec3};
use nalgebra::Matrix3;

/// Returns a vector along the Z-axis with the given Z-value.
pub fn zvec(z: f64) -> DVec3 {
    dvec3(0.0, 0.0, z)
}

/// Rotates a vector 90 degrees counterclockwise.
pub fn rotate_90_degrees(vector: DVec2) -> DVec2 {
    dvec2(-vector.y, vector.x)
}

/// A line segment in 2D space.
pub struct LineSegment {
    pub start: DVec2,
    pub end: DVec2,
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
    /// Creates a plane from a point on the plane and a normal vector.
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

    /// Returns a point on the plane.
    pub fn point(&self) -> DVec3 {
        self.point
    }

    /// Projects a vector to the plane.
    pub fn project_vector(&self, vector: DVec3) -> DVec3 {
        vector - self.normal.dot(vector) * self.normal
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
    (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x) >= 0.0
}

/// An ellipse in 2D space.
pub struct Ellipse {
    axis: DVec2,
    position: DVec2,
    a: f64,
    b: f64,
}

impl Ellipse {
    /// Creates a new ellipse from the given normalized first axis, center, `a` and `b` values.
    ///
    /// # Panics
    ///
    /// Panics if the given axis is not normalized.
    pub fn new(axis: DVec2, position: DVec2, a: f64, b: f64) -> Self {
        assert!(axis.is_normalized());

        Self {
            axis,
            position,
            a,
            b,
        }
    }

    /// Calculates the four tangents tangents betwen `self` and `other`.
    ///
    /// Returns for each tangent the coefficients of the equation `ax + by + c = 0` describing it.
    pub fn tangents_to(&self, other: &Ellipse) -> [DVec3; 4] {
        let c1 = self.matrix_representation().inverse();
        let c2 = other.matrix_representation().inverse();

        let (eigenvalues, eigenvectors) = eigendecomposition(c2.inverse() * c1);
        let transposed_eigenvectors = eigenvectors.transpose();
        let l1 = transposed_eigenvectors * (c1 - eigenvalues.x * c2) * eigenvectors;
        let l2 = transposed_eigenvectors * (c1 - eigenvalues.y * c2) * eigenvectors;

        let x0 = (-l2.z_axis.z / l2.x_axis.x).sqrt();
        let y0 = (-l1.z_axis.z / l1.y_axis.y).sqrt();

        [
            dvec3(x0, y0, 1.0),
            dvec3(x0, -y0, 1.0),
            dvec3(-x0, y0, 1.0),
            dvec3(-x0, -y0, 1.0),
        ]
        .map(|solution| eigenvectors * solution)
    }

    /// Calculates the matrix representation of the ellipse.
    fn matrix_representation(&self) -> DMat3 {
        let a_squared = self.a * self.a;
        let b_squared = self.b * self.b;
        let diagonal = DMat3::from_diagonal(dvec3(a_squared, b_squared, -a_squared * b_squared));

        let rotation = DMat2::from_cols(self.axis, rotate_90_degrees(self.axis)).transpose();
        let transform = DMat3::from_mat2(rotation) * DMat3::from_translation(-self.position);

        transform.transpose() * diagonal * transform
    }
}

/// Calculates the eigendecomposition of a diagonalizeable matrix.
#[allow(clippy::many_single_char_names)]
fn eigendecomposition(matrix: DMat3) -> (DVec3, DMat3) {
    let matrix = Matrix3::from_column_slice(&matrix.to_cols_array());
    let (unitary_matrix, triangular_matrix) = matrix.schur().unpack();

    let [a, 0.0, 0.0, b, d, 0.0, c, e, f] = *triangular_matrix.as_slice() else {
        panic!("the matrix should be triangular")
    };

    // The eigenvalues are the diagonal elements of the triangular matrix
    let eigenvalues = dvec3(a, d, f);
    // The eigenvectors are the eigenvectors of the triangular matrix multiplied by the unitary matrix
    let eigenvectors = DMat3::from_cols_slice(unitary_matrix.as_slice())
        * DMat3 {
            x_axis: DVec3::X,
            y_axis: dvec3(b, d - a, 0.0),
            z_axis: dvec3(c * (d - f) - b * e, e * (a - f), (f - d) * (a - f)),
        };

    (eigenvalues, eigenvectors)
}
