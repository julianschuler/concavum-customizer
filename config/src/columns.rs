use glam::DVec2;
use serde::{de::Error as DeserializeError, Deserialize, Deserializer, Serialize};
use show::{egui::Ui, Show};

use crate::{CurvatureAngle, FiniteFloat, SideAngle, Vec2};

/// A per column configuration for the finger cluster keys.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Columns {
    /// The left side column of the finger cluster keys.
    pub left_side_column: SideColumn,
    /// The normal columns of the finger cluster keys.
    pub normal_columns: NormalColumns,
    /// The right side column of the finger cluster keys.
    pub right_side_column: SideColumn,
}

impl Show for Columns {
    fn show(&mut self, ui: &mut Ui) {}
}

/// A per column configuration for the finger cluster keys.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Serialize, PartialEq, Eq, Hash)]
pub struct NormalColumns(Vec<NormalColumn>);

impl NormalColumns {
    /// Returns the first normal column.
    pub fn first(&self) -> &NormalColumn {
        self.0.first().expect("there has to be at least one row")
    }

    /// Returns the last normal column.
    pub fn last(&self) -> &NormalColumn {
        self.0.last().expect("there has to be at least one row")
    }
}

impl<'de> Deserialize<'de> for NormalColumns {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner: Vec<NormalColumn> = Vec::deserialize(deserializer)?;

        if inner.is_empty() {
            return Err(D::Error::custom(
                "invalid value: normal columns must not be empty",
            ));
        }

        Ok(Self(inner))
    }
}

/// A configuration of a normal finger cluster column.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NormalColumn {
    /// The column curvature as an angle between two neighboring keys.
    pub curvature_angle: CurvatureAngle,
    /// The offset of the column in Y and Z.
    pub offset: Vec2<FiniteFloat>,
}

/// A configuration of a side column.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SideColumn {
    /// Whether the side column is active.
    pub active: bool,
    /// The angle of the side column to the neighboring normal one.
    pub side_angle: SideAngle,
}

/// A type of a column.
pub enum ColumnType {
    /// A normal column.
    Normal,
    /// A left side column.
    LeftSide,
    /// A right side column.
    RightSide,
}

impl ColumnType {
    /// Returns the side factor of the column type.
    #[must_use]
    pub fn side(&self) -> f64 {
        match self {
            ColumnType::Normal => 0.0,
            ColumnType::LeftSide => 1.0,
            ColumnType::RightSide => -1.0,
        }
    }
}

/// The config of a single finger cluster column.
pub struct ColumnConfig {
    /// The type of the column.
    pub column_type: ColumnType,
    /// The curvature angle of the column.
    pub curvature_angle: f64,
    /// The offset of the column in X and Y.
    pub offset: DVec2,
    /// The side angle of the column.
    pub side_angle: f64,
}

impl From<&Columns> for Vec<ColumnConfig> {
    fn from(columns: &Columns) -> Self {
        let mut configs = Vec::new();

        if columns.left_side_column.active {
            let &NormalColumn {
                curvature_angle,
                offset,
            } = columns.normal_columns.first();

            configs.push(ColumnConfig {
                column_type: ColumnType::LeftSide,
                curvature_angle: curvature_angle.into(),
                offset: offset.into(),
                side_angle: columns.left_side_column.side_angle.into(),
            });
        }

        configs.extend(columns.normal_columns.0.iter().map(|normal_column| {
            let &NormalColumn {
                curvature_angle,
                offset,
            } = normal_column;

            ColumnConfig {
                column_type: ColumnType::Normal,
                curvature_angle: curvature_angle.into(),
                offset: offset.into(),
                side_angle: 0.0,
            }
        }));

        if columns.right_side_column.active {
            let &NormalColumn {
                curvature_angle,
                offset,
            } = columns.normal_columns.last();

            configs.push(ColumnConfig {
                column_type: ColumnType::RightSide,
                curvature_angle: curvature_angle.into(),
                offset: offset.into(),
                side_angle: columns.left_side_column.side_angle.into(),
            });
        }

        configs
    }
}
