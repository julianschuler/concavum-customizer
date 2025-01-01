//! The `config` crate contains everything related to the available configuration options.

mod columns;
mod primitives;

use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use show::{
    egui::{DragValue, Frame, Margin, ScrollArea, Ui},
    Show,
};
use show_derive::Show;

pub use columns::{ColumnConfig, ColumnType, Columns, NormalColumn, SideColumn};
pub use primitives::{
    ColumnCurvatureAngle, FiniteFloat, PositiveFloat, Ranged, SideAngle, ThumbCurvatureAngle, Vec2,
    Vec3,
};
pub use show::Color;

/// A configuration of a keyboard.
#[derive(Clone, Serialize, Deserialize, Eq)]
pub struct Config {
    /// The preview configuration.
    pub preview: Preview,
    /// The finger cluster configuration.
    pub finger_cluster: FingerClusterWrapper,
    /// The thumb cluster configuration.
    pub thumb_cluster: ThumbCluster,
    /// The keyboard configuration.
    pub keyboard: Keyboard,
    /// The colors of the keyboard.
    pub colors: Colors,
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
        self.finger_cluster == other.finger_cluster
            && self.thumb_cluster == other.thumb_cluster
            && self.keyboard == other.keyboard
    }
}

impl Hash for Config {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.finger_cluster.hash(state);
        self.thumb_cluster.hash(state);
        self.keyboard.hash(state);
    }
}

impl Show for Config {
    fn show(&mut self, ui: &mut Ui) -> bool {
        const MARGIN: Margin = Margin {
            left: 0.0,
            right: 8.0,
            top: 4.0,
            bottom: 8.0,
        };

        let mut changed = false;

        ScrollArea::vertical().show(ui, |ui| {
            Frame::default().inner_margin(MARGIN).show(ui, |ui| {
                changed |= self.preview.show_section(ui);
                changed |= self.finger_cluster.show_section(ui);
                changed |= self.thumb_cluster.show_section(ui);
                changed |= self.keyboard.show_section(ui);
                changed |= self.colors.show_section(ui);
            })
        });

        changed
    }
}

/// A configuration for previewing a keyboard.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default, Serialize, Deserialize, Show, PartialEq, Eq, Hash)]
pub struct Preview {
    /// Whether to show the keys.
    pub show_keys: bool,
    /// Whether to show the interface PCB.
    pub show_matrix_pcb: bool,
    /// Whether to show the interface PCB.
    pub show_interface_pcb: bool,
    /// Whether to show the keyboard.
    pub show_keyboard: bool,
    /// Whether to show the bottom plate.
    pub show_bottom_plate: bool,
}

#[derive(Clone, Serialize, PartialEq, Eq, Hash)]
/// A wrapper struct for the actual finger cluster configuration.
pub struct FingerClusterWrapper(FingerCluster);

impl Deref for FingerClusterWrapper {
    type Target = FingerCluster;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for FingerClusterWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut finger_cluster = FingerCluster::deserialize(deserializer)?;

        let home_row_index = i8::from(finger_cluster.home_row_index);
        let maximum_index = i8::from(finger_cluster.rows) - 1;

        if home_row_index >= 1 && home_row_index <= maximum_index {
            finger_cluster.home_row_index.set_maximum(maximum_index);

            Ok(Self(finger_cluster))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: home row index `{home_row_index}` is not between 1 and {maximum_index}",
            )))
        }
    }
}

impl Show for FingerClusterWrapper {
    const TITLE: &'static str = FingerCluster::TITLE;

    fn show(&mut self, ui: &mut Ui) -> bool {
        let changed = self.0.show(ui);

        if changed {
            self.0.home_row_index.set_maximum(i8::from(self.0.rows) - 1);
        }

        changed
    }
}

/// A configuration of a finger cluster.
#[derive(Clone, Serialize, Deserialize, Show, PartialEq, Eq, Hash)]
pub struct FingerCluster {
    /// The number of rows.
    pub rows: Ranged<i8, 2, 5>,
    /// The settings per column.
    pub columns: Columns,
    /// The distance between two neighboring keys in X and Y direction.
    pub key_distance: Vec2<PositiveFloat>,
    /// The row index of the home row (usually 1).
    pub home_row_index: HomeRowIndex,
}

/// An index of the home row.
#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct HomeRowIndex {
    inner: i8,
    #[serde(skip)]
    maximum: i8,
}

impl HomeRowIndex {
    fn set_maximum(&mut self, maximum: i8) {
        self.maximum = maximum;
        self.inner = self.inner.min(maximum);
    }
}

impl From<HomeRowIndex> for i8 {
    fn from(value: HomeRowIndex) -> Self {
        value.inner
    }
}

impl Show for HomeRowIndex {
    fn show(&mut self, ui: &mut Ui) -> bool {
        ui.add(DragValue::new(&mut self.inner).clamp_range(1..=self.maximum))
            .changed()
    }
}

/// A configuration of a thumb cluster.
#[derive(Clone, Serialize, Deserialize, Show, PartialEq, Eq, Hash)]
pub struct ThumbCluster {
    /// The number of thumb keys.
    pub keys: Ranged<i8, 1, 6>,
    /// The thumb well curvature as an angle between two neighboring keys.
    pub curvature_angle: ThumbCurvatureAngle,
    /// The rotation of the thumb cluster in relation to the finger cluster.
    pub rotation: Vec3<FiniteFloat>,
    /// The offset of the thumb cluster in relation to the finger cluster.
    pub offset: Vec3<FiniteFloat>,
    /// The distance between two neighboring thumb keys.
    pub key_distance: PositiveFloat,
    /// The index of the key the thumb is naturally resting on (usually the center key).
    pub resting_key_index: Ranged<i8, 0, 5>,
}

/// A configuration of other keyboard settings.
#[derive(Clone, Serialize, Deserialize, Show, PartialEq, Eq, Hash)]
pub struct Keyboard {
    /// The keyboard tilting angle along X and Y
    pub tilting_angle: Vec2<FiniteFloat>,
    /// The space around the key cluster.
    pub circumference_distance: PositiveFloat,
    /// The rounding radius of the top keyboard edges.
    pub rounding_radius: FiniteFloat,
    /// The wall thickness of the keyboard shell.
    pub shell_thickness: PositiveFloat,
    /// The thickness of the bottom plate.
    pub bottom_plate_thickness: PositiveFloat,
    /// The resolution used for meshing, size of the smallest feature.
    pub resolution: PositiveFloat,
}

/// A configuration of the colors used for displaying the keyboard.
#[derive(Clone, Default, Serialize, Deserialize, Show, PartialEq, Eq, Hash)]
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
    /// The color of the FFC connector.
    pub ffc_connector: Color,
    /// The color of the background.
    pub background: Color,
}

/// An error type for configuration parsing errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A float is not finite.
    #[error("float is not finite")]
    NonFiniteFloat,
    /// A float is not positive.
    #[error("float is not positive")]
    NonPositiveFloat,
    /// A value is out of range.
    #[error("value is out of range")]
    OutOfRangeValue,
}
