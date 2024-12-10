use std::ops::{Add, Mul, Neg, Sub};

use serde::Serialize;

/// The conversion factor between an i32 value and the units of length or rotation.
pub const VALUE_TO_UNIT: i32 = 1_000_000;

/// A unit of length.
#[derive(Serialize, Clone, Copy, Default)]
#[serde(transparent)]
pub struct Length(i32);

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

impl Mul<Length> for f32 {
    type Output = Length;

    fn mul(self, rhs: Length) -> Self::Output {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Length((self * rhs.0 as f32) as i32)
    }
}

impl Neg for Length {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

/// A unit of rotation.
#[derive(Serialize, Clone, Copy, Default)]
#[serde(transparent)]
pub struct Angle(i32);

impl Angle {
    /// Calculates the sine and cosine of the angle.
    pub fn sin_cos(self) -> (f32, f32) {
        #[allow(clippy::cast_precision_loss)]
        f32::sin_cos((self.0 as f32).to_radians())
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self((self.0 + rhs.0) % (360 * VALUE_TO_UNIT))
    }
}

impl Sub for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self((self.0 - rhs.0) % (360 * VALUE_TO_UNIT))
    }
}

impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

pub trait IntoUnit {
    /// Returns a length with the value of `self` in millimeters.
    fn mm(self) -> Length;

    /// Returns an angle with the value of `self` in degree.
    fn deg(self) -> Angle;

    /// Returns an angle with the value of `self` in radians.
    fn rad(self) -> Angle;
}

impl IntoUnit for i32 {
    fn mm(self) -> Length {
        Length(self * VALUE_TO_UNIT)
    }

    fn deg(self) -> Angle {
        Angle(self * VALUE_TO_UNIT)
    }

    fn rad(self) -> Angle {
        f64::from(self).rad()
    }
}

impl IntoUnit for f32 {
    fn mm(self) -> Length {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Length((self * VALUE_TO_UNIT as f32) as i32)
    }

    fn deg(self) -> Angle {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Angle((self * VALUE_TO_UNIT as f32) as i32)
    }

    fn rad(self) -> Angle {
        self.to_degrees().deg()
    }
}

impl IntoUnit for f64 {
    fn mm(self) -> Length {
        #[allow(clippy::cast_possible_truncation)]
        Length((self * f64::from(VALUE_TO_UNIT)) as i32)
    }

    fn deg(self) -> Angle {
        #[allow(clippy::cast_possible_truncation)]
        Angle((self * f64::from(VALUE_TO_UNIT)) as i32)
    }

    fn rad(self) -> Angle {
        self.to_degrees().deg()
    }
}
