use std::hash::{Hash, Hasher};

use glam::{DVec2, DVec3};
use serde::{de::Error as DeserializeError, Deserialize, Deserializer, Serialize};

use crate::Error;

/// A 2-dimensional vector.
#[derive(Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
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

/// A 3-dimensional vector.
#[derive(Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
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

/// A finite 64-bit floating point type.
#[derive(Copy, Clone, Serialize, Default, PartialEq)]
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
#[derive(Copy, Clone, Serialize, Default, PartialEq, Eq, Hash)]
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

impl<'de> Deserialize<'de> for PositiveFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = FiniteFloat::deserialize(deserializer)?;

        if inner.0 > 0.0 {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: `{}` is not greater than 0.0",
                inner.0
            )))
        }
    }
}

/// A range constrained 64-bit floating point type.
#[derive(Copy, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct Ranged<const LOWER: i8, const UPPER: i8>(FiniteFloat);

impl<const LOWER: i8, const UPPER: i8> From<Ranged<LOWER, UPPER>> for f64 {
    fn from(ranged: Ranged<LOWER, UPPER>) -> Self {
        ranged.0.into()
    }
}

impl<const LOWER: i8, const UPPER: i8> TryFrom<f64> for Ranged<LOWER, UPPER> {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let inner = FiniteFloat::try_from(value)?;

        if inner.0 >= f64::from(LOWER) && inner.0 <= f64::from(UPPER) {
            Ok(Self(inner))
        } else {
            Err(Error::OutOfRangeFloat)
        }
    }
}

impl<'de, const LOWER: i8, const UPPER: i8> Deserialize<'de> for Ranged<LOWER, UPPER> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = FiniteFloat::deserialize(deserializer)?;

        if inner.0 >= f64::from(LOWER) && inner.0 <= f64::from(UPPER) {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "invalid value: `{}` is not between {LOWER} and {UPPER}",
                inner.0
            )))
        }
    }
}
