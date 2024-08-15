use serde::Serialize;

use crate::unit::Unit;

/// A 2-dimensional point.
#[derive(Serialize, Clone, Copy)]
pub struct Point(Unit, Unit);

impl Point {
    /// Creates a new point from the given coordinates.
    pub fn new(x: Unit, y: Unit) -> Self {
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
