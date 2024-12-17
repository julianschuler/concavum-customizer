use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use glam::{vec2, Vec2};
use serde::Serialize;

use crate::unit::{Angle, IntoUnit, Length};

/// A 2-dimensional point.
#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
pub struct Point(Length, Length);

impl Point {
    /// Creates a new point from the given coordinates.
    pub const fn new(x: Length, y: Length) -> Self {
        Self(x, y)
    }

    /// Returns the X coordinate of the position.
    pub fn x(self) -> Length {
        self.0
    }

    /// Returns the Y coordinate of the position.
    pub fn y(self) -> Length {
        self.1
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}

impl Sub for Point {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        vec2((self.x() - rhs.x()).into(), (self.y() - rhs.y()).into())
    }
}

impl Add<Vec2> for Point {
    type Output = Point;

    fn add(self, rhs: Vec2) -> Self::Output {
        Point::new(self.x() + rhs.x.mm(), self.y() + rhs.y.mm())
    }
}

impl Sub<Vec2> for Point {
    type Output = Point;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Point::new(self.x() - rhs.x.mm(), self.y() - rhs.y.mm())
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
    pub const fn new(x: Length, y: Length, angle: Option<Angle>) -> Self {
        Self(x, y, angle)
    }

    /// Returns the X coordinate of the position.
    pub fn x(self) -> Length {
        self.0
    }

    /// Returns the Y coordinate of the position.
    pub fn y(self) -> Length {
        self.1
    }

    /// Returns the angle of the position.
    pub fn angle(self) -> Option<Angle> {
        self.2
    }

    /// Returns the corresponding point without the angle.
    pub fn point(self) -> Point {
        Point::new(self.x(), self.y())
    }

    /// Applies the affine transform given by `self` to the given position.
    pub fn transform_position(self, other: Position) -> Self {
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

    /// Applies the affine transform given by `self` the the given point.
    pub fn transform_point(self, point: Point) -> Point {
        let (sin, cos) = self.angle().unwrap_or_default().sin_cos();

        let x = self.x() + cos * point.x() + sin * point.y();
        let y = self.y() - sin * point.x() + cos * point.y();

        Point::new(x, y)
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        self.transform_position(rhs)
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        self.transform_position(-rhs)
    }
}

impl Add<Point> for Position {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        self.transform_point(rhs)
    }
}

impl Sub<Point> for Position {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        self.transform_point(-rhs)
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
