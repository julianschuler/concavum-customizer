use std::{
    fmt::{Display, Formatter, Result as FormatResult},
    hash::{Hash, Hasher},
};

use glam::{DVec2, DVec3};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use show::{
    egui::{DragValue, Ui},
    Show,
};

use crate::Error;

const DRAG_SPEED: f64 = 0.1;

/// A curvature angle between two neighboring column keys.
pub type ColumnCurvatureAngle = Ranged<FiniteFloat, 0, 50>;

/// A curvature angle between two neighboring thumb keys.
pub type ThumbCurvatureAngle = Ranged<FiniteFloat, -20, 50>;

/// A side angle between two columns.
pub type SideAngle = Ranged<FiniteFloat, 0, 30>;

/// A 2-dimensional vector.
#[derive(Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
pub struct Vec2<T> {
    /// The X component of the vector.
    pub x: T,
    /// The Y component of the vector.
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

impl<T: Serialize + Copy> Serialize for Vec2<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        [self.x, self.y].serialize(serializer)
    }
}

impl<T: Show> Show for Vec2<T> {
    fn show(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Reverse order since widgets are placed right to left
            changed |= self.y.show(ui);
            changed |= self.x.show(ui);
        });

        changed
    }
}

/// A 3-dimensional vector.
#[derive(Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
pub struct Vec3<T> {
    /// The X component of the vector.
    pub x: T,
    /// The Y component of the vector.
    pub y: T,
    /// The Z component of the vector.
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

impl<T: Serialize + Copy> Serialize for Vec3<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        [self.x, self.y, self.z].serialize(serializer)
    }
}

impl<T: Show> Show for Vec3<T> {
    fn show(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Reverse order since widgets are placed right to left
            changed |= self.z.show(ui);
            changed |= self.y.show(ui);
            changed |= self.x.show(ui);
        });

        changed
    }
}

/// A finite 64-bit floating point type.
#[derive(Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(try_from = "f64")]
pub struct FiniteFloat(f64);

impl Eq for FiniteFloat {}

impl Hash for FiniteFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl From<FiniteFloat> for f64 {
    fn from(float: FiniteFloat) -> Self {
        float.0
    }
}

impl TryFrom<f64> for FiniteFloat {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(Error::NonFiniteFloat)
        }
    }
}

impl Display for FiniteFloat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        self.0.fmt(f)
    }
}

impl Show for FiniteFloat {
    fn show(&mut self, ui: &mut Ui) -> bool {
        let mut value = f64::from(*self);

        let changed = ui
            .add(
                DragValue::new(&mut value)
                    .clamp_range(f64::MIN..=f64::MAX)
                    .speed(DRAG_SPEED),
            )
            .changed();

        *self = value.try_into().expect("value should be finite");

        changed
    }
}

/// A strictly positive finite 64-bit floating point type.
#[derive(Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
#[serde(try_from = "f64")]
pub struct PositiveFloat(FiniteFloat);

impl From<PositiveFloat> for f64 {
    fn from(float: PositiveFloat) -> Self {
        float.0.into()
    }
}

impl TryFrom<f64> for PositiveFloat {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let inner = FiniteFloat::try_from(value)?;

        if inner.0 > 0.0 {
            Ok(Self(inner))
        } else {
            Err(Error::NonPositiveFloat)
        }
    }
}

impl Show for PositiveFloat {
    fn show(&mut self, ui: &mut Ui) -> bool {
        let mut value = f64::from(*self);

        let changed = ui
            .add(
                DragValue::new(&mut value)
                    .clamp_range(DRAG_SPEED..=f64::MAX)
                    .speed(DRAG_SPEED),
            )
            .changed();

        *self = value
            .try_into()
            .expect("value should be finite and positive");

        changed
    }
}

/// A range constrained value.
#[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub struct Ranged<T, const LOWER: i8, const UPPER: i8>(T);

impl<const LOWER: i8, const UPPER: i8> From<Ranged<FiniteFloat, LOWER, UPPER>> for f64 {
    fn from(ranged: Ranged<FiniteFloat, LOWER, UPPER>) -> Self {
        ranged.0.into()
    }
}

impl<const LOWER: i8, const UPPER: i8> From<Ranged<i8, LOWER, UPPER>> for i8 {
    fn from(ranged: Ranged<i8, LOWER, UPPER>) -> Self {
        ranged.0
    }
}

impl<const LOWER: i8, const UPPER: i8> TryFrom<FiniteFloat> for Ranged<FiniteFloat, LOWER, UPPER> {
    type Error = Error;

    fn try_from(value: FiniteFloat) -> Result<Self, Self::Error> {
        if value.0 >= f64::from(LOWER) && value.0 <= f64::from(UPPER) {
            Ok(Self(value))
        } else {
            Err(Error::OutOfRangeValue)
        }
    }
}

impl<const LOWER: i8, const UPPER: i8> TryFrom<f64> for Ranged<FiniteFloat, LOWER, UPPER> {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        FiniteFloat::try_from(value)?.try_into()
    }
}

impl<const LOWER: i8, const UPPER: i8> TryFrom<i8> for Ranged<i8, LOWER, UPPER> {
    type Error = Error;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        if value >= LOWER && value <= UPPER {
            Ok(Self(value))
        } else {
            Err(Error::OutOfRangeValue)
        }
    }
}

impl<'de, T, const LOWER: i8, const UPPER: i8> Deserialize<'de> for Ranged<T, LOWER, UPPER>
where
    T: Clone + Display + Deserialize<'de> + TryInto<Ranged<T, LOWER, UPPER>>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;

        value.clone().try_into().map_err(|_| {
            D::Error::custom(format!(
                "invalid value: `{value}` is not between {LOWER} and {UPPER}",
            ))
        })
    }
}

impl<const LOWER: i8, const UPPER: i8> Show for Ranged<FiniteFloat, LOWER, UPPER> {
    fn show(&mut self, ui: &mut Ui) -> bool {
        let mut value = f64::from(*self);

        let changed = ui
            .add(
                DragValue::new(&mut value)
                    .clamp_range(LOWER..=UPPER)
                    .speed(DRAG_SPEED),
            )
            .changed();

        *self = value.try_into().expect("value should be within range");

        changed
    }
}

impl<const LOWER: i8, const UPPER: i8> Show for Ranged<i8, LOWER, UPPER> {
    fn show(&mut self, ui: &mut Ui) -> bool {
        ui.add(DragValue::new(&mut self.0).clamp_range(LOWER..=UPPER))
            .changed()
    }
}
