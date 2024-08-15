use std::ops::Deref;

use serde::Serialize;

/// The conversion factor between an i32 value and the units of length or rotation.
pub const VALUE_TO_UNIT: i32 = 1_000_000;

/// A unit of length.
#[derive(Serialize, Clone, Copy, Default)]
#[serde(transparent)]
pub struct Length(i32);

impl Deref for Length {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A unit of rotation.
#[derive(Serialize, Clone, Copy)]
#[serde(transparent)]
pub struct Angle(i32);

impl Deref for Angle {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait IntoUnit {
    /// Returns a length with the value of `self` in millimeters.
    fn mm(self) -> Length;

    /// Returns an angle with the value of `self` in degree.
    fn deg(self) -> Angle;
}

impl IntoUnit for i32 {
    fn mm(self) -> Length {
        Length(self * VALUE_TO_UNIT)
    }

    fn deg(self) -> Angle {
        Angle(self * VALUE_TO_UNIT)
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
}
