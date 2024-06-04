use std::{fs::read_to_string, io, num::NonZeroU8, ops::Deref, path::Path};

use glam::{DVec2, DVec3};
use hex_color::HexColor;
use serde::{de::Error as DeserializeError, Deserialize, Deserializer};

pub const EPSILON: f64 = 0.001;
pub const KEY_CLEARANCE: f64 = 1.0;
pub const CURVATURE_HEIGHT: f64 = 6.6;

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

#[derive(Clone, Deserialize)]
pub struct Preview {
    pub show_keys: bool,
    pub show_interface_pcb: bool,
    pub show_bottom_plate: bool,
    pub resolution: PositiveFloat,
    pub light_positions: Vec<Vec3<FiniteFloat>>,
}

#[derive(Deserialize)]
pub struct FingerCluster {
    pub rows: NonZeroU8,
    pub columns: Columns,
    pub key_distance: Vec2<PositiveFloat>,
    pub home_row_index: u8,
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Column {
    Normal {
        curvature_angle: CurvatureAngle,
        offset: Vec2<FiniteFloat>,
    },
    Side {
        side_angle: SideAngle,
    },
}

#[derive(Deserialize)]
pub struct ThumbCluster {
    pub keys: NonZeroU8,
    pub curvature_angle: CurvatureAngle,
    pub rotation: Vec3<FiniteFloat>,
    pub offset: Vec3<FiniteFloat>,
    pub key_distance: PositiveFloat,
    pub resting_key_index: u8,
}

#[derive(Deserialize)]
pub struct Keyboard {
    pub tilting_angle: Vec2<FiniteFloat>,
    pub circumference_distance: PositiveFloat,
    pub rounding_radius: FiniteFloat,
    pub shell_thickness: PositiveFloat,
    pub bottom_plate_thickness: PositiveFloat,
}

#[derive(Clone, Deserialize)]
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

/// A 2-dimensional vector
#[derive(Copy, Clone, Deserialize)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Into<f64>> From<Vec2<T>> for DVec2 {
    fn from(value: Vec2<T>) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

/// A 3-dimensional vector
#[derive(Copy, Clone, Deserialize)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Into<f64>> From<Vec3<T>> for DVec3 {
    fn from(value: Vec3<T>) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
            z: value.z.into(),
        }
    }
}

/// A finite 64-bit floating point type.
#[derive(Copy, Clone)]
pub struct FiniteFloat(f64);

impl From<FiniteFloat> for f64 {
    fn from(float: FiniteFloat) -> Self {
        float.0
    }
}

impl<'de> Deserialize<'de> for FiniteFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = f64::deserialize(deserializer)?;

        if inner.is_finite() {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: `{inner}` is not finite"
            )))
        }
    }
}

/// A strictly positive finite 64-bit floating point type.
#[derive(Copy, Clone)]
pub struct PositiveFloat(f64);

impl From<PositiveFloat> for f64 {
    fn from(float: PositiveFloat) -> Self {
        float.0
    }
}

impl<'de> Deserialize<'de> for PositiveFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = FiniteFloat::deserialize(deserializer)?.0;

        if inner > 0.0 {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: `{inner}` is not greater than 0.0"
            )))
        }
    }
}

#[derive(Copy, Clone)]
pub struct Ranged<const LOWER: i8, const UPPER: i8>(f64);

impl<const LOWER: i8, const UPPER: i8> Deref for Ranged<LOWER, UPPER> {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de, const LOWER: i8, const UPPER: i8> Deserialize<'de> for Ranged<LOWER, UPPER> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = FiniteFloat::deserialize(deserializer)?.0;

        if inner >= f64::from(LOWER) && inner <= f64::from(UPPER) {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: `{inner}` is not between {LOWER} and {UPPER}"
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
