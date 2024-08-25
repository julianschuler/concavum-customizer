use glam::{dvec2, dvec3, DAffine3, DMat3, DVec2, DVec3};

use crate::{
    geometry::{Line as GeometricLine, Plane},
    key_positions::{Column, KeyPositions},
    util::SideY,
};

/// The size of the PCB pads underneath each key.
pub const PAD_SIZE: DVec2 = dvec2(13.0, 14.0);
/// The thickness of the matrix PCB.
pub const THICKNESS: f64 = 0.6;
/// The width of the connectors between keys and columns.
pub const CONNECTOR_WIDTH: f64 = 2.0;

const SWITCH_HEIGHT: f64 = 5.0;

/// A PCB connecting the keys to each other in a matrix.
pub struct MatrixPcb {
    /// The key connectors between keys in the columns.
    pub key_connectors: Vec<KeyConnectors>,
}

impl MatrixPcb {
    /// Creates a new matrix PCB from the given key positions.
    pub fn from_positions(positions: &KeyPositions) -> Self {
        let key_connectors = positions
            .columns
            .iter()
            .map(KeyConnectors::from_column)
            .collect();

        Self { key_connectors }
    }
}

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

/// A connector between keys.
pub enum KeyConnector {
    /// An arc.
    Arc(UpwardsArc),
    /// A line.
    Line(Line),
}

impl KeyConnector {
    /// Creates a new key connector from the given column.
    #[must_use]
    fn from_column(column: &Column) -> Self {
        if let Some(next_position) = column.get(1) {
            let position = column.first();
            let start_point = key_connector_point(*position, SideY::Top);
            let end_point = key_connector_point(*next_position, SideY::Bottom);

            let plane = Plane::new(start_point, position.y_axis);
            let line = GeometricLine::new(end_point, next_position.z_axis);

            let angle = position
                .z_axis
                .dot(next_position.z_axis)
                .clamp(-1.0, 1.0)
                .acos();

            if angle == 0.0 {
                let length = start_point.distance(end_point);

                Self::Line(Line { length })
            } else {
                let intersection = plane.intersection(&line).unwrap_or_default();
                let radius = (intersection - start_point).dot(position.z_axis);

                Self::Arc(UpwardsArc { radius, angle })
            }
        } else {
            Self::Line(Line { length: 1.0 })
        }
    }
}

impl Segment for KeyConnector {
    fn positions(&self) -> Vec<DAffine3> {
        match self {
            KeyConnector::Arc(arc) => arc.positions(),
            KeyConnector::Line(line) => line.positions(),
        }
    }

    fn length(&self) -> f64 {
        match self {
            KeyConnector::Arc(arc) => arc.length(),
            KeyConnector::Line(line) => line.length(),
        }
    }
}

/// The connectors between keys in a column.
pub struct KeyConnectors {
    /// The connector itself.
    pub connector: KeyConnector,
    /// The positions of the connector.
    pub positions: Vec<DAffine3>,
}

impl KeyConnectors {
    /// Creates new key connectors for a given column.
    #[must_use]
    fn from_column(column: &Column) -> Self {
        let positions = column
            .windows(2)
            .flat_map(|window| {
                let position = window[0];
                let next_position = window[1];

                let transformed_next_key = position
                    .inverse()
                    .transform_point3(next_position.translation);

                let left_x_offset =
                    transformed_next_key.x.max(0.0) - (PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0;
                let right_x_offset =
                    transformed_next_key.x.min(0.0) + (PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0;

                let start_point = key_connector_point(position, SideY::Top);
                let left_position = DAffine3 {
                    matrix3: position.matrix3,
                    translation: start_point + left_x_offset * position.x_axis,
                };
                let right_position = DAffine3 {
                    matrix3: position.matrix3,
                    translation: start_point + right_x_offset * position.x_axis,
                };

                [left_position, right_position]
            })
            .collect();
        let connector = KeyConnector::from_column(column);

        Self {
            connector,
            positions,
        }
    }
}

fn key_connector_point(position: DAffine3, side: SideY) -> DVec3 {
    position.translation + side.direction() * PAD_SIZE.y / 2.0 * position.y_axis
        - (SWITCH_HEIGHT + THICKNESS / 2.0) * position.z_axis
}

/// A cubic Bézier curve.
pub struct BezierCurve {
    start: DAffine3,
    control1: DVec3,
    control2: DVec3,
    end: DAffine3,
}

impl BezierCurve {
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
