use glam::vec2;
use model::matrix_pcb::{PAD_SIZE, ROUTER_BIT_DIAMETER};

use crate::{
    kicad_pcb::KicadPcb,
    matrix_pcb::{
        features::{Column, Features},
        AddPath,
    },
    point, position,
    primitives::{Point, Position},
    unit::Length,
};

/// Panelizes the given PCB using the given features as reference.
pub fn panelize(pcb: &mut KicadPcb, features: &Features) {
    let bounding_box = BoundingBox::from_features(features);

    let upper_positions = features
        .columns
        .iter()
        .map(|column| column.last() + position!(0, -PAD_SIZE.y / 2.0, None));
    let upper_rail = Rail::upper(&bounding_box, upper_positions);

    // upper_rail.add_outline(pcb);
}

/// A bounding box for the matrix PCB.
struct BoundingBox {
    min: Point,
    max: Point,
}

impl BoundingBox {
    /// Creates a new bounding box from the given matrix PCB features.
    pub fn from_features(features: &Features) -> Self {
        let points = features
            .columns
            .iter()
            .flat_map(Column::positions)
            .chain(features.thumb_switches.positions())
            .flat_map(|&position| {
                [
                    position + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
                    position + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
                    position + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
                    position + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
                ]
            });

        Self::from_points(points)
    }

    /// Creates a new bounding box from the given points.
    pub fn from_points(points: impl IntoIterator<Item = Point>) -> Self {
        let (min, max) = points
            .into_iter()
            .fold((Point::MAX, Point::MIN), |(min, max), point| {
                (min.min(point), max.max(point))
            });

        Self { min, max }
    }
}

struct Rail {
    upper: bool,
    tabs: Vec<Tab>,
    left_corner: Point,
    length: Length,
    text: &'static str,
}

impl Rail {
    /// The width of a rail.
    const WIDTH: Length = Length::new(5.0);

    /// Creates the upper rail from the given bounding box and tab positions.
    pub fn upper(
        bounding_box: &BoundingBox,
        tab_positions: impl IntoIterator<Item = Position>,
    ) -> Self {
        let left_corner = point!(
            bounding_box.min.x(),
            bounding_box.min.y() - ROUTER_BIT_DIAMETER.into()
        );
        let length = bounding_box.max.x() - bounding_box.min.x();
        let tabs: Vec<_> = tab_positions
            .into_iter()
            .map(|position| Tab::new(position, left_corner.y()))
            .collect();

        Self {
            upper: true,
            tabs,
            left_corner,
            length,
            text: "Open Source Hardware - Automatically generated",
        }
    }

    /// Adds the outline of the rail to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        let sign = if self.upper { -1 } else { 1 };

        let outer_left_corner = self.left_corner + vec2(0.0, (sign * Self::WIDTH).into());
        let outer_right_corner =
            self.left_corner + vec2(self.length.into(), (sign * Self::WIDTH).into());
        let outline_points: Vec<_> = [outer_left_corner, self.left_corner]
            .into_iter()
            .chain(self.tabs.iter().flat_map(Tab::corners))
            .chain([
                self.left_corner + vec2(self.length.into(), 0.0),
                outer_right_corner,
            ])
            .collect();

        for chunk in outline_points.chunks(4) {
            pcb.add_outline_path(chunk);
        }
        pcb.add_outline_path(&[outer_left_corner, outer_right_corner]);

        pcb.add_zone(outline_points.clone());
    }

    /// Adds the mousebites for separating the rail from the Tabs and
    pub fn add_mousebites(&self, pcb: &mut KicadPcb) {}
}

struct Tab {
    position: Position,
    rail_y: Length,
}

impl Tab {
    /// The width of a Tab.
    const WIDTH: Length = Length::new(10.0);

    /// Creates a new tab at the given position extending to a rail with the given Y coordinate.
    pub fn new(position: Position, rail_y: Length) -> Self {
        Self { position, rail_y }
    }

    /// Returns the lower left corner of the tab.
    pub fn lower_left_corner(&self) -> Point {
        point!(self.position.x() - Self::WIDTH / 2, self.rail_y)
    }

    /// Returns the lower right corner of the tab.
    pub fn lower_right_corner(&self) -> Point {
        point!(self.position.x() + Self::WIDTH / 2, self.rail_y)
    }

    /// Returns the upper left corner of the tab.
    pub fn upper_left_corner(&self) -> Point {
        let projection_factor = 1.0 / self.position.angle().map_or(1.0, |angle| angle.sin_cos().1);

        self.position + point!(-projection_factor * Self::WIDTH / 2, 0)
    }

    /// Returns the upper right corner of the tab.
    pub fn upper_right_corner(&self) -> Point {
        let projection_factor = 1.0 / self.position.angle().map_or(1.0, |angle| angle.sin_cos().1);

        self.position + point!(projection_factor * Self::WIDTH / 2, 0)
    }

    pub fn add_mousebites(&self, pcb: &mut KicadPcb) {}

    pub fn corners(&self) -> [Point; 4] {
        let projection_factor = 1.0 / self.position.angle().map_or(1.0, |angle| angle.sin_cos().1);

        [
            point!(self.position.x() - Self::WIDTH / 2, self.rail_y),
            self.position + point!(-projection_factor * Self::WIDTH / 2, 0),
            self.position + point!(projection_factor * Self::WIDTH / 2, 0),
            point!(self.position.x() + Self::WIDTH / 2, self.rail_y),
        ]
    }
}
