use serde::Serialize;

use crate::unit::{Angle, Length};

/// A 2-dimensional point.
#[derive(Serialize, Clone, Copy)]
pub struct Point(Length, Length);

impl Point {
    /// Creates a new point from the given coordinates.
    pub fn new(x: Length, y: Length) -> Self {
        Self(x, y)
    }
}

/// Creates a point from the given coordinates in millimeter.
#[macro_export]
macro_rules! point {
    ($x:expr, $y:expr) => {
        $crate::primitives::Point::new(($x).mm(), ($y).mm())
    };
}

/// A size of an object as width and height.
#[derive(Serialize, Clone, Copy)]
pub struct Size(Length, Length);

impl Size {
    /// Creates a new size from the given width and height.
    pub fn new(width: Length, height: Length) -> Self {
        Self(width, height)
    }
}

/// Creates a size tuple from the given values in millimeter.
#[macro_export]
macro_rules! size {
    ($width:expr, $height:expr) => {
        $crate::primitives::Size::new(($width).mm(), ($height).mm())
    };
}

/// A 2-dimensional position with an optional orientation angle.
#[derive(Serialize, Clone, Copy)]
pub struct Position(Length, Length, Option<Angle>);

impl Position {
    /// Creates a new position from the given coordinates and optional angle.
    pub fn new(x: Length, y: Length, angle: Option<Angle>) -> Self {
        Self(x, y, angle)
    }

    /// Returns the angle of the position.
    pub fn angle(&self) -> Option<Angle> {
        self.2
    }
}

/// Creates a position from the given values in millimeter and the angle.
#[macro_export]
macro_rules! position {
    ($x:expr, $y:expr, $angle:expr) => {
        $crate::primitives::Position::new(($x).mm(), ($y).mm(), $angle)
    };
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
