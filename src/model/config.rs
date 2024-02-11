use std::{fs::read_to_string, io, ops::Deref, path::Path};

use glam::{Vec2, Vec3A};
use hex_color::HexColor;
use serde::{de::Error as DeserializeError, Deserialize, Deserializer};

pub const EPSILON: f32 = 0.001;
pub const KEY_CLEARANCE: f32 = 1.0;

pub type CurvatureAngle = Ranged<-20, 50>;
pub type SideAngle = Ranged<0, 30>;

#[derive(Deserialize)]
pub struct Config {
    pub preview: Preview,
    pub finger_cluster: FingerCluster,
    pub thumb_cluster: ThumbCluster,
    pub keyboard: Keyboard,
    pub colors: Colors,
}

#[derive(Deserialize)]
pub struct Preview {
    pub show_keys: bool,
    pub show_interface_pcb: bool,
    pub show_bottom_plate: bool,
    pub mesh_resolution: PositiveFloat,
    pub light_positions: Vec<Vec3A>,
}

#[derive(Deserialize)]
pub struct FingerCluster {
    pub rows: PositiveInt,
    pub columns: Columns,
    pub key_distance: [PositiveFloat; 2],
    pub home_row_index: u8,
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Column {
    Normal {
        curvature_angle: CurvatureAngle,
        offset: Vec2,
    },
    Side {
        side_angle: SideAngle,
    },
}

#[derive(Deserialize)]
pub struct ThumbCluster {
    pub keys: PositiveInt,
    pub curvature_angle: CurvatureAngle,
    pub rotation: Vec3A,
    pub offset: Vec3A,
    pub key_distance: PositiveFloat,
    pub resting_key_index: u8,
}

#[derive(Deserialize)]
pub struct Keyboard {
    pub tilting_angle: Vec2,
    pub circumference_distance: PositiveFloat,
    pub shell_thickness: PositiveFloat,
    pub bottom_plate_thickness: PositiveFloat,
}

#[derive(Deserialize)]
pub struct Colors {
    pub keyboard: HexColor,
    pub keycap: HexColor,
    pub switch: HexColor,
    pub matrix_pcb: HexColor,
    pub interface_pcb: HexColor,
    pub fpc_connector: HexColor,
    pub background: HexColor,
}

pub struct Columns(Vec<Column>);

impl Deref for Columns {
    type Target = Vec<Column>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Columns {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner: Vec<Column> = Vec::deserialize(deserializer)?;

        if inner.is_empty() {
            return Err(D::Error::custom("invalid value: columns must not be empty"));
        }

        if inner.iter().enumerate().any(|(i, column)| {
            i != 0 && i != inner.len() - 1 && matches!(column, Column::Side { .. })
        }) {
            return Err(D::Error::custom(
                "invalid value: only the first and last column can have a side angle",
            ));
        }

        if !inner
            .iter()
            .any(|column| matches!(column, Column::Normal { .. }))
        {
            return Err(D::Error::custom(
                "invalid value: there has to be at least one column with curvature and offset",
            ));
        }

        Ok(Self(inner))
    }
}

/// Strictly positive 8-bit unsigned integer type.
#[derive(Copy, Clone)]
pub struct PositiveInt(u8);

impl Deref for PositiveInt {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for PositiveInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = u8::deserialize(deserializer)?;

        if inner > 0 {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: {inner} is not greater than 0"
            )))
        }
    }
}

/// Strictly positive 64-bit floating point type.
#[derive(Copy, Clone)]
pub struct PositiveFloat(f32);

impl Deref for PositiveFloat {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for PositiveFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = f32::deserialize(deserializer)?;

        if inner > 0.0 {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: {inner} is not greater than 0.0"
            )))
        }
    }
}

pub struct PositiveDVec2 {
    pub x: f32,
    pub y: f32,
}

impl From<&[PositiveFloat; 2]> for PositiveDVec2 {
    fn from(value: &[PositiveFloat; 2]) -> Self {
        let [x, y] = *value;
        Self { x: *x, y: *y }
    }
}

#[derive(Copy, Clone)]
pub struct Ranged<const LOWER: i8, const UPPER: i8>(f32);

impl<const LOWER: i8, const UPPER: i8> Deref for Ranged<LOWER, UPPER> {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de, const LOWER: i8, const UPPER: i8> Deserialize<'de> for Ranged<LOWER, UPPER> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = f32::deserialize(deserializer)?;

        if inner >= f32::from(LOWER) && inner <= f32::from(UPPER) {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: {inner} is not between {LOWER} and {UPPER}"
            )))
        }
    }
}

impl Config {
    pub fn try_from_path(config_path: &Path) -> Result<Self, Error> {
        Ok(toml::from_str(&read_to_string(config_path)?)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to open file
    #[error("failed to open file")]
    FileOpen(#[from] io::Error),
    /// Failed to parse TOML
    #[error("failed to parse TOML")]
    TomlParse(#[from] toml::de::Error),
}
