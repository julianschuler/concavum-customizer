//! The config crate contains everything related to the available configuration options.

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

/// A color.
pub type Color = HexColor;

/// A curvature angle between two neighboring keys.
pub type CurvatureAngle = Ranged<-20, 50>;

/// A side angle between two columns.
pub type SideAngle = Ranged<0, 30>;

/// A configuration of a keyboard.
#[derive(Clone, Serialize, Deserialize, Eq)]
pub struct Config {
    /// The preview configuration.
    pub preview: Preview,
    /// The finger cluster configuration.
    pub finger_cluster: FingerCluster,
    /// The thumb cluster configuration.
    pub thumb_cluster: ThumbCluster,
    /// The keyboard configuration.
    pub keyboard: Keyboard,
    /// The colors of the keyboard.
    pub colors: Colors,
}

/// A configuration for previewing a keyboard.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Preview {
    /// Whether to show the keys during preview.
    pub show_keys: bool,
    /// Whether to show the interface PCB during preview.
    pub show_interface_pcb: bool,
    /// Whether to show the bottom plate during preview.
    pub show_bottom_plate: bool,
    /// The resolution used for meshing, size of the smallest feature.
    pub resolution: PositiveFloat,
    /// The light positions, will be hardcoded at a later stage.
    pub light_positions: Vec<Vec3<FiniteFloat>>,
}

/// A configuration of a finger cluster.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FingerCluster {
    /// The number of rows, automatic PCB generation is supported for 1 to 5 rows.
    pub rows: NonZeroU8,
    /// The settings per column, automatic PCB generation is supported for 2 to 6 columns.
    pub columns: Columns,
    /// The distance between two neighboring keys in X and Y direction.
    pub key_distance: Vec2<PositiveFloat>,
    /// The row index of the home row (usually 1).
    pub home_row_index: u8,
}

/// A configuration of a thumb cluster.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ThumbCluster {
    /// The number of thumb keys, automatic PCB generation is supported for 1 to 6 keys.
    pub keys: NonZeroU8,
    /// The thumb well curvature as an angle between two neighboring keys.
    pub curvature_angle: CurvatureAngle,
    /// The rotation of the thumb cluster in relation to the finger cluster.
    pub rotation: Vec3<FiniteFloat>,
    /// The offset of the thumb cluster in relation to the finger cluster.
    pub offset: Vec3<FiniteFloat>,
    /// The distance between two neighboring thumb keys.
    pub key_distance: PositiveFloat,
    /// The index of the key the thumb is naturally resting on (usually the center key).
    pub resting_key_index: u8,
}

/// A configuration of other keyboard settings.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Keyboard {
    /// The keyboard tilting angle along X and Y
    pub tilting_angle: Vec2<FiniteFloat>,
    /// The space around the key cluster.
    pub circumference_distance: PositiveFloat,
    /// The rounding radius of the top keyboard edges.
    pub rounding_radius: FiniteFloat,
    /// The wall thickness of the keyboard shell.
    pub shell_thickness: PositiveFloat,
    /// The thickness of the bottom plate
    pub bottom_plate_thickness: PositiveFloat,
}

/// A configuration of the colors used for displaying the keyboard.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Colors {
    /// The color of the keyboard.
    pub keyboard: Color,
    /// The color of the keycaps.
    pub keycap: Color,
    /// The color of the switches.
    pub switch: Color,
    /// The color of the matrix PCB.
    pub matrix_pcb: Color,
    /// The color of the interface PCB.
    pub interface_pcb: Color,
    /// The color of the FPC connector.
    pub fpc_connector: Color,
    /// The color of the background.
    pub background: Color,
}

/// A per column configuration for the finger cluster keys.
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

/// A configuration of a single finger cluster column.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged, deny_unknown_fields)]
pub enum Column {
    /// A configuration of a normal column.
    Normal {
        /// The column curvature as an angle between two neighboring keys.
        curvature_angle: CurvatureAngle,
        /// The offset of the column in Y and Z.
        offset: Vec2<FiniteFloat>,
    },
    /// A configuration of a side column.
    Side {
        /// The angle of the side column to the neighboring normal one.
        side_angle: SideAngle,
    },
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

/// The error type for errors regarding parsing configurations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to open file.
    #[error("failed to open file")]
    FileOpen(#[from] io::Error),
    /// Failed to parse TOML.
    #[error("failed to parse TOML")]
    TomlParse(#[from] toml::de::Error),
    /// Float is not finite.
    #[error("float is not finite")]
    NonFiniteFloat,
    /// Float is not positive.
    #[error("float is not positive")]
    NonPositiveFloat,
    /// Float is out of range.
    #[error("float is out of range")]
    OutOfRangeFloat,
}
