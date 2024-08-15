use serde::Serialize;

use crate::unit::Length;

/// A 2-dimensional point.
#[derive(Serialize, Clone, Copy)]
pub struct Point(Length, Length);

impl Point {
    /// Creates a new point from the given coordinates.
    pub fn new(x: Length, y: Length) -> Self {
        Self(x, y)
    }
}

/// A Universally Unique Identifier (UUID).
pub struct Uuid(uuid::Uuid);

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Uuid {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
