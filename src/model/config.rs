use std::{fs::read_to_string, io, ops::Deref, path::Path};

use glam::{DVec2, DVec3};
use hex_color::HexColor;
use serde::{de::Error as DeserializeError, Deserialize, Deserializer};

pub const EPSILON: f64 = 0.001;

pub type CurvatureAngle = Ranged<-20, 50>;

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
    pub triangulation_tolerance: Positive,
    pub light_directions: Vec<DVec3>,
}

#[derive(Deserialize)]
pub struct FingerCluster {
    pub rows: u8,
    pub columns: Vec<Column>,
    pub side_angles: (CurvatureAngle, CurvatureAngle),
    pub key_distance: [Positive; 2],
    pub home_row_index: u8,
}

#[derive(Deserialize)]
pub struct Column {
    pub curvature_angle: CurvatureAngle,
    pub offset: DVec2,
}

#[derive(Deserialize)]
pub struct ThumbCluster {
    pub keys: u8,
    pub curvature_angle: Ranged<-20, 45>,
    pub rotation: DVec3,
    pub offset: DVec3,
    pub key_distance: Positive,
}

#[derive(Deserialize)]
pub struct Keyboard {
    pub tilting_angle: DVec2,
    pub shell_thickness: Positive,
    pub bottom_plate_thickness: Positive,
}

#[derive(Deserialize)]
pub struct Colors {
    pub keycap: HexColor,
    pub switch: HexColor,
    pub matrix_pcb: HexColor,
    pub interface_pcb: HexColor,
    pub fpc_connector: HexColor,
    pub background: HexColor,
}

#[derive(Copy, Clone)]
pub struct Positive(f64);

impl Deref for Positive {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Positive {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = f64::deserialize(deserializer)?;

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
    pub x: f64,
    pub y: f64,
}

impl From<&[Positive; 2]> for PositiveDVec2 {
    fn from(value: &[Positive; 2]) -> Self {
        let [x, y] = *value;
        Self { x: *x, y: *y }
    }
}

impl Config {
    pub fn try_from_path(config_path: &Path) -> Result<Self, Error> {
        Ok(toml::from_str(&read_to_string(config_path)?)?)
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
        let inner = f64::deserialize(deserializer)?;

        if inner >= LOWER as f64 && inner <= UPPER as f64 {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: {inner} is not between {LOWER} and {UPPER}"
            )))
        }
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
