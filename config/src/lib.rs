mod primitives;

use std::{
    hash::{Hash, Hasher},
    io,
    num::NonZeroU8,
    ops::Deref,
};

use hex_color::HexColor;
use serde::{de::Error as DeserializeError, Deserialize, Deserializer, Serialize};

pub use primitives::{FiniteFloat, PositiveFloat, Ranged, Vec2, Vec3};

pub type Color = HexColor;
pub type CurvatureAngle = Ranged<-20, 50>;
pub type SideAngle = Ranged<0, 30>;

#[derive(Clone, Serialize, Deserialize, Eq)]
pub struct Config {
    pub preview: Preview,
    pub finger_cluster: FingerCluster,
    pub thumb_cluster: ThumbCluster,
    pub keyboard: Keyboard,
    pub colors: Colors,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Preview {
    pub show_keys: bool,
    pub show_interface_pcb: bool,
    pub show_bottom_plate: bool,
    pub resolution: PositiveFloat,
    pub light_positions: Vec<Vec3<FiniteFloat>>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FingerCluster {
    pub rows: NonZeroU8,
    pub columns: Columns,
    pub key_distance: Vec2<PositiveFloat>,
    pub home_row_index: u8,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ThumbCluster {
    pub keys: NonZeroU8,
    pub curvature_angle: CurvatureAngle,
    pub rotation: Vec3<FiniteFloat>,
    pub offset: Vec3<FiniteFloat>,
    pub key_distance: PositiveFloat,
    pub resting_key_index: u8,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Keyboard {
    pub tilting_angle: Vec2<FiniteFloat>,
    pub circumference_distance: PositiveFloat,
    pub rounding_radius: FiniteFloat,
    pub shell_thickness: PositiveFloat,
    pub bottom_plate_thickness: PositiveFloat,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Colors {
    pub keyboard: Color,
    pub keycap: Color,
    pub switch: Color,
    pub matrix_pcb: Color,
    pub interface_pcb: Color,
    pub fpc_connector: Color,
    pub background: Color,
}

#[derive(Clone, Serialize, PartialEq, Eq, Hash)]
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

impl Default for Config {
    fn default() -> Self {
        let toml_string = include_str!("default.toml");
        toml::from_str(toml_string).expect("default configuration should always be deserializable")
    }
}

// Exclude fields independent from the calculated mesh from Hash and PartialEq
impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.preview.resolution == other.preview.resolution
            && self.finger_cluster == other.finger_cluster
            && self.thumb_cluster == other.thumb_cluster
            && self.keyboard == other.keyboard
    }
}

impl Hash for Config {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.preview.resolution.hash(state);
        self.finger_cluster.hash(state);
        self.thumb_cluster.hash(state);
        self.keyboard.hash(state);
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
    /// Float is not finite
    #[error("float is not finite")]
    NonFiniteFloat,
    /// Float is not positive
    #[error("float is not positive")]
    NonPositiveFloat,
    /// Float is out of range
    #[error("float is out of range")]
    OutOfRangeFloat,
}
