use std::f64::consts::{FRAC_PI_2, PI};

use glam::{DAffine3, DMat3, DVec2, DVec3, Vec3Swizzles};

use crate::{
    key_positions::{Column, ColumnType, ThumbKeys},
    matrix_pcb::{
        segments::{Arc, BezierCurve, Line, Segment},
        CONNECTOR_WIDTH, PAD_SIZE, SWITCH_HEIGHT, THICKNESS,
    },
    util::{SideX, SideY},
};

/// A connector between keys.
pub enum KeyConnector {
    /// An arc.
    Arc(Arc),
    /// A line.
    Line(Line),
}

impl KeyConnector {
    /// Creates a new key connector from a the given offset in Y and Z.
    #[must_use]
    pub fn new(offset: DVec2) -> Self {
        let angle = 2.0 * offset.to_angle();

        if angle == 0.0 {
            Self::Line(Line::new(offset.x))
        } else {
            let radius = offset.length_squared() / (2.0 * offset.y);

            Self::Arc(Arc::new(radius, angle, DVec3::X))
        }
    }

    /// Creates a new key connector from the given column.
    #[must_use]
    fn from_column(column: &Column) -> Self {
        if let Some(&next_position) = column.get(1) {
            let position = column.first();
            let start_point = vertical_connector_point(position, SideY::Top);
            let end_point = vertical_connector_point(next_position, SideY::Bottom);

            let direction = position.matrix3.inverse() * (end_point - start_point);

            Self::new(direction.yz())
        } else {
            Self::Line(Line::new(1.0))
        }
    }

    /// Creates new key connectors for the given thumb keys.
    #[must_use]
    fn from_thumb_keys(thumb_keys: &ThumbKeys) -> Self {
        if let Some(&next_position) = thumb_keys.get(1) {
            let position = thumb_keys.first();
            let start_point = horizontal_connector_point(position, SideX::Right);
            let end_point = horizontal_connector_point(next_position, SideX::Left);

            let direction = position.matrix3.inverse() * (end_point - start_point);

            Self::new(direction.xz())
        } else {
            Self::Line(Line::new(1.0))
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
#[allow(clippy::module_name_repetitions)]
pub struct KeyConnectors {
    /// The connector itself.
    pub connector: KeyConnector,
    /// The positions of the connector.
    pub positions: Vec<DAffine3>,
}

impl KeyConnectors {
    /// Creates new key connectors for a given column.
    #[must_use]
    pub fn from_column(column: &Column) -> Self {
        let positions = column
            .windows(2)
            .flat_map(|window| {
                let bottom_position = window[0];
                let top_position = window[1];

                let transformed_next_key = bottom_position
                    .inverse()
                    .transform_point3(top_position.translation);

                let left_x_offset =
                    transformed_next_key.x.max(0.0) - (PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0;
                let right_x_offset =
                    transformed_next_key.x.min(0.0) + (PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0;

                let start_point = vertical_connector_point(bottom_position, SideY::Top);
                let left_position = DAffine3 {
                    matrix3: bottom_position.matrix3,
                    translation: start_point + left_x_offset * bottom_position.x_axis,
                };
                let right_position = DAffine3 {
                    matrix3: bottom_position.matrix3,
                    translation: start_point + right_x_offset * bottom_position.x_axis,
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

    /// Creates new key connectors for the given thumb keys.
    #[must_use]
    pub fn from_thumb_keys(thumb_keys: &ThumbKeys) -> Self {
        let positions = thumb_keys
            .windows(2)
            .flat_map(|window| {
                let left_position = window[0];
                let matrix3 = left_position.matrix3 * DMat3::from_rotation_z(-FRAC_PI_2);

                let start_point = horizontal_connector_point(left_position, SideX::Right);
                let bottom_position = DAffine3 {
                    matrix3,
                    translation: start_point
                        - (PAD_SIZE.y - CONNECTOR_WIDTH) / 2.0 * left_position.y_axis,
                };
                let top_position = DAffine3 {
                    matrix3,
                    translation: start_point
                        + (PAD_SIZE.y - CONNECTOR_WIDTH) / 2.0 * left_position.y_axis,
                };

                [bottom_position, top_position]
            })
            .collect();
        let connector = KeyConnector::from_thumb_keys(thumb_keys);

        Self {
            connector,
            positions,
        }
    }
}

/// A connector between two neighboring columns.
pub enum ColumnConnector {
    /// A connector between two normal columns.
    Normal(NormalColumnConnector),
    /// A connector between a normal and a side column.
    Side(SideColumnConnector),
}

impl ColumnConnector {
    /// Creates a new connector between two neighboring columns.
    #[must_use]
    pub fn from_columns(
        left_column: &Column,
        right_column: &Column,
        home_row_index: usize,
    ) -> Self {
        let left_position = left_column[home_row_index];
        let right_position = right_column[home_row_index];

        match (left_column.column_type, right_column.column_type) {
            (ColumnType::Normal, ColumnType::Normal) => Self::Normal(
                NormalColumnConnector::from_positions(left_position, right_position),
            ),
            _ => Self::Side(SideColumnConnector::from_positions(
                left_position,
                right_position,
            )),
        }
    }
}

/// A connector between two normal columns.
pub struct NormalColumnConnector {
    /// The Bézier curve segment in the center of the connector.
    pub bezier_curve: BezierCurve,
    /// The radius of the top and bottom arcs.
    pub arc_radius: f64,
    /// The side of the left arc.
    pub left_arc_side: SideY,
}

impl NormalColumnConnector {
    /// Creates a new normal column connector from the given key positions.
    #[must_use]
    fn from_positions(left_position: DAffine3, right_position: DAffine3) -> Self {
        let transformed_right_translation = left_position
            .inverse()
            .transform_point3(right_position.translation);

        let arc_radius = (transformed_right_translation.x - PAD_SIZE.x) / 2.0;
        let left_arc_side = if transformed_right_translation.z >= 0.0 {
            SideY::Top
        } else {
            SideY::Bottom
        };

        let start_position =
            normal_connector_position(left_position, arc_radius, SideX::Right, left_arc_side);
        let end_position = normal_connector_position(
            right_position,
            arc_radius,
            SideX::Left,
            left_arc_side.opposite(),
        );

        let bezier_curve = BezierCurve::from_positions(start_position, end_position);

        Self {
            bezier_curve,
            arc_radius,
            left_arc_side,
        }
    }
}

impl Segment for NormalColumnConnector {
    fn positions(&self) -> Vec<DAffine3> {
        let arc_positions = Arc::new(
            self.arc_radius,
            FRAC_PI_2,
            self.left_arc_side.direction() * DVec3::NEG_Z,
        )
        .positions();

        let bezier_positions = self.bezier_curve.positions();
        let first_arc_position = bezier_positions
            .first()
            .copied()
            .expect("there should always be a position")
            * DAffine3::from_rotation_z(PI);
        let second_arc_position = bezier_positions
            .last()
            .copied()
            .expect("there should always be a position");

        arc_positions
            .iter()
            .rev()
            .map(|&position| first_arc_position * position)
            .chain(bezier_positions)
            .chain(
                arc_positions
                    .iter()
                    .map(|&position| second_arc_position * position),
            )
            .collect()
    }

    fn length(&self) -> f64 {
        self.bezier_curve.length()
    }
}

/// A connector between a normal and a side column.
pub struct SideColumnConnector {
    /// The connector itself.
    pub connector: KeyConnector,
    /// The position of the connector.
    pub position: DAffine3,
}

impl SideColumnConnector {
    /// Creates a new side column connector from the given key positions.
    fn from_positions(left_position: DAffine3, right_position: DAffine3) -> Self {
        let start_point = horizontal_connector_point(left_position, SideX::Right);
        let end_point = horizontal_connector_point(right_position, SideX::Left);

        let direction = left_position.matrix3.inverse() * (end_point - start_point);
        let connector = KeyConnector::new(direction.xz());

        let position = DAffine3 {
            matrix3: left_position.matrix3 * DMat3::from_rotation_z(-FRAC_PI_2),
            translation: start_point,
        };

        Self {
            connector,
            position,
        }
    }
}

impl Segment for SideColumnConnector {
    fn positions(&self) -> Vec<DAffine3> {
        self.connector.positions()
    }

    fn length(&self) -> f64 {
        self.connector.length()
    }
}

fn vertical_connector_point(position: DAffine3, side: SideY) -> DVec3 {
    position.translation + side.direction() * PAD_SIZE.y / 2.0 * position.y_axis
        - (SWITCH_HEIGHT + THICKNESS / 2.0) * position.z_axis
}

fn horizontal_connector_point(position: DAffine3, side: SideX) -> DVec3 {
    position.translation + side.direction() * PAD_SIZE.x / 2.0 * position.x_axis
        - (SWITCH_HEIGHT + THICKNESS / 2.0) * position.z_axis
}

fn normal_connector_position(
    position: DAffine3,
    arc_radius: f64,
    side_x: SideX,
    side_y: SideY,
) -> DAffine3 {
    let translation = position.translation
        + side_x.direction() * (PAD_SIZE.x / 2.0 + arc_radius) * position.x_axis
        + side_y.direction()
            * ((PAD_SIZE.y - CONNECTOR_WIDTH) / 2.0 - arc_radius)
            * position.y_axis
        - (SWITCH_HEIGHT + THICKNESS / 2.0) * position.z_axis;

    DAffine3 {
        matrix3: position.matrix3,
        translation,
    }
}
