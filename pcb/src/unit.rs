use std::ops::{Add, Div, Mul, Neg, Sub};

use serde::Serialize;

/// The conversion factor between an i32 value and the units of length or rotation.
pub const VALUE_TO_UNIT: i32 = 1_000_000;
/// The maxium angle value.
const MAXIMUM_ANGLE: i32 = 360 * VALUE_TO_UNIT;

/// A unit of length.
#[derive(Serialize, Clone, Copy, Default, PartialEq, Eq)]
#[serde(transparent)]
pub struct Length(i32);

impl Length {
    /// Returns a length with the given value in millimeters.
    pub const fn new(value: f64) -> Length {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Length((value * VALUE_TO_UNIT as f64) as i32)
    }
}

impl Add for Length {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Length {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<Length> for i32 {
    type Output = Length;

    fn mul(self, rhs: Length) -> Self::Output {
        Length(self * rhs.0)
    }
}

impl Mul<Length> for f32 {
    type Output = Length;

    fn mul(self, rhs: Length) -> Self::Output {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Length((self * rhs.0 as f32) as i32)
    }
}

impl Div<i32> for Length {
    type Output = Length;

    fn div(self, rhs: i32) -> Self::Output {
        Length(self.0 / rhs)
    }
}

impl Neg for Length {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl From<i32> for Length {
    fn from(value: i32) -> Self {
        Length(value * VALUE_TO_UNIT)
    }
}

impl From<f32> for Length {
    fn from(value: f32) -> Self {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Length((value * VALUE_TO_UNIT as f32) as i32)
    }
}

impl From<f64> for Length {
    fn from(value: f64) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Length((value * f64::from(VALUE_TO_UNIT)) as i32)
    }
}

impl From<Length> for f32 {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: Length) -> Self {
        value.0 as f32 / VALUE_TO_UNIT as f32
    }
}

impl From<Length> for f64 {
    fn from(value: Length) -> Self {
        f64::from(value.0) / f64::from(VALUE_TO_UNIT)
    }
}

/// A unit of rotation.
#[derive(Serialize, Clone, Copy, Default)]
#[serde(transparent)]
pub struct Angle(i32);

impl Angle {
    /// Calculates the sine and cosine of the angle.
    pub fn sin_cos(self) -> (f32, f32) {
        f32::from(self).to_radians().sin_cos()
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self((self.0 + rhs.0) % MAXIMUM_ANGLE)
    }
}

impl Sub for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self((self.0 - rhs.0) % MAXIMUM_ANGLE)
    }
}

impl Mul<Angle> for i32 {
    type Output = Angle;

    fn mul(self, rhs: Angle) -> Self::Output {
        Angle((self * rhs.0) % MAXIMUM_ANGLE)
    }
}

impl Div<i32> for Angle {
    type Output = Angle;

    fn div(self, rhs: i32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl From<Angle> for f32 {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: Angle) -> Self {
        value.0 as f32 / VALUE_TO_UNIT as f32
    }
}

impl From<Angle> for f64 {
    fn from(value: Angle) -> Self {
        f64::from(value.0) / f64::from(VALUE_TO_UNIT)
    }
}

/// A trait for converting values into angles.
pub trait IntoAngle {
    /// Returns an angle with the value of `self` in degree.
    fn deg(self) -> Angle;

    /// Returns an angle with the value of `self` in radians.
    fn rad(self) -> Angle;
}

impl IntoAngle for i32 {
    fn deg(self) -> Angle {
        Angle(self * VALUE_TO_UNIT)
    }

    fn rad(self) -> Angle {
        f64::from(self).rad()
    }
}

impl IntoAngle for f32 {
    fn deg(self) -> Angle {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Angle((self * VALUE_TO_UNIT as f32) as i32)
    }

    fn rad(self) -> Angle {
        self.to_degrees().deg()
    }
}

impl IntoAngle for f64 {
    fn deg(self) -> Angle {
        #[allow(clippy::cast_possible_truncation)]
        Angle((self * f64::from(VALUE_TO_UNIT)) as i32)
    }

    fn rad(self) -> Angle {
        self.to_degrees().deg()
    }
}
