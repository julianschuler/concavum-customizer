use std::ops::Deref;

use serde::Serialize;

/// The conversion factor between millimiters and the internal unit of length.
pub const MM_TO_UNIT: i32 = 1_000_000;

/// An internal unit of length.
#[derive(Serialize, Clone, Copy, Default)]
#[serde(transparent)]
pub struct Unit(i32);

impl Deref for Unit {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait IntoUnit {
    /// Returns a unit with the length of `self` in millimeters.
    fn mm(self) -> Unit;
}

impl IntoUnit for f32 {
    fn mm(self) -> Unit {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Unit((self * MM_TO_UNIT as f32) as i32)
    }
}
