use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

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
        $crate::primitives::Point::new(
            $crate::unit::IntoUnit::mm($x),
            $crate::unit::IntoUnit::mm($y),
        )
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
        $crate::primitives::Size::new(
            $crate::unit::IntoUnit::mm($width),
            $crate::unit::IntoUnit::mm($height),
        )
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

    /// Returns the X coordinate of the position.
    pub fn x(&self) -> Length {
        self.0
    }

    /// Returns the Y coordinate of the position.
    pub fn y(&self) -> Length {
        self.1
    }

    /// Returns the angle of the position.
    pub fn angle(&self) -> Option<Angle> {
        self.2
    }

    /// Applies the affine transform given by the other position to `self`.
    ///
    /// The transform is applied in the reference frame of `self`.
    pub fn affine(&self, other: Position) -> Self {
        let (sin, cos) = self.angle().unwrap_or_default().sin_cos();

        let x = self.x() + cos * other.x() + sin * other.y();
        let y = self.y() - sin * other.x() + cos * other.y();

        let angle = if let Some(angle) = self.2 {
            Some(angle + other.angle().unwrap_or_default())
        } else {
            other.angle()
        };

        Self::new(x, y, angle)
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        self.affine(rhs)
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        self.affine(-rhs)
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Position {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Neg for Position {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, self.2.map(|angle| -angle))
    }
}

/// Creates a position from the given values in millimeter and the angle.
#[macro_export]
macro_rules! position {
    ($x:expr, $y:expr, $angle:expr) => {
        $crate::primitives::Position::new(
            $crate::unit::IntoUnit::mm($x),
            $crate::unit::IntoUnit::mm($y),
            $angle,
        )
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
