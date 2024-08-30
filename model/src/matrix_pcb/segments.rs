use glam::{dvec3, DAffine3, DMat3, DVec3};

/// A trait defining a segment.
pub trait Segment {
    /// The positions of the segment.
    fn positions(&self) -> Vec<DAffine3>;

    /// The length of the segment.
    fn length(&self) -> f64;
}

/// An arc along the Y-axis curving upwards.
pub struct UpwardsArc {
    radius: f64,
    angle: f64,
}

impl UpwardsArc {
    /// Creates a new upwards arc.
    pub fn new(radius: f64, angle: f64) -> Self {
        Self { radius, angle }
    }
}

impl Segment for UpwardsArc {
    fn positions(&self) -> Vec<DAffine3> {
        let maximum_angle = 3.0_f64.to_radians();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let segments = ((self.angle / maximum_angle) as u32).max(1);

        let translation = DAffine3::from_translation(dvec3(0.0, 0.0, self.radius));

        (0..=segments)
            .map(|segment| {
                #[allow(clippy::cast_lossless)]
                let angle = segment as f64 / segments as f64 * self.angle;

                translation * DAffine3::from_rotation_x(angle) * translation.inverse()
            })
            .collect()
    }

    fn length(&self) -> f64 {
        self.radius * self.angle
    }
}

/// A line along the Y-axis.
pub struct Line {
    length: f64,
}

impl Line {
    /// Creates a new line with the given length.
    pub fn new(length: f64) -> Self {
        Self { length }
    }
}

impl Segment for Line {
    fn positions(&self) -> Vec<DAffine3> {
        [
            DAffine3::IDENTITY,
            DAffine3::from_translation(dvec3(0.0, self.length, 0.0)),
        ]
        .to_vec()
    }

    fn length(&self) -> f64 {
        self.length
    }
}

/// A cubic Bézier curve.
pub struct BezierCurve {
    start: DAffine3,
    control1: DVec3,
    control2: DVec3,
    end: DAffine3,
}

impl BezierCurve {
    /// Creates a cubic Bézier with a low maximum curvature from the given positions.
    pub fn from_positions(start: DAffine3, end: DAffine3) -> BezierCurve {
        let distance = start.inverse().transform_point3(end.translation).y / 2.0;

        Self {
            start,
            control1: start.translation + distance * start.y_axis,
            control2: end.translation - distance * end.y_axis,
            end,
        }
    }

    /// Returns the point corresponding to the given value of `t`.
    fn parametric_point(&self, t: f64) -> DVec3 {
        let p1 = lerp(self.start.translation, self.control1, t);
        let p2 = lerp(self.control1, self.control2, t);
        let p3 = lerp(self.control2, self.end.translation, t);
        let p4 = lerp(p1, p2, t);
        let p5 = lerp(p2, p3, t);

        lerp(p4, p5, t)
    }

    /// Returns the tangent corresponding to the given value of `t`.
    fn parametric_tangent(&self, t: f64) -> DVec3 {
        let t_squared = t * t;

        self.start.translation * (-3.0 * t_squared + 6.0 * t - 3.0)
            + self.control1 * (9.0 * t_squared - 12.0 * t + 3.0)
            + self.control2 * (-9.0 * t_squared + 6.0 * t)
            + self.end.translation * (3.0 * t_squared)
    }
}

impl Segment for BezierCurve {
    fn positions(&self) -> Vec<DAffine3> {
        const SEGMENTS: usize = 50;

        (0..=SEGMENTS)
            .map(|index| {
                #[allow(clippy::cast_precision_loss)]
                let t = index as f64 / SEGMENTS as f64;

                let tangent = self.parametric_tangent(t);
                let up = lerp(self.start.z_axis, self.end.z_axis, t);

                let y_axis = tangent.normalize();
                let x_axis = up.cross(y_axis).normalize();
                let z_axis = x_axis.cross(y_axis);

                let translation = self.parametric_point(t);

                DAffine3 {
                    matrix3: DMat3 {
                        x_axis,
                        y_axis,
                        z_axis,
                    },
                    translation,
                }
            })
            .collect()
    }

    fn length(&self) -> f64 {
        self.positions()
            .windows(2)
            .map(|window| window[0].translation.distance(window[1].translation))
            .sum()
    }
}

/// Computes the linear interpolation between `a` and `b` with
/// `lerp(a, b, 0) = a` and `lerp(a, b, 1) = b`.
fn lerp(a: DVec3, b: DVec3, t: f64) -> DVec3 {
    (1.0 - t) * a + t * b
}
