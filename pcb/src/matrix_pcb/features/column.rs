use std::iter::once;

use model::matrix_pcb::{ColumnKeyConnectors, Segment, PAD_SIZE};

use crate::{position, primitives::Position};

/// The positions of a column of finger key switches.
pub struct Column {
    home_switch: Position,
    switches_below: Vec<Position>,
    switches_above: Vec<Position>,
}

impl Column {
    /// Creates a new column from the corresponding key connectors and home switch position.
    pub fn from_key_connectors(
        key_connectors: &ColumnKeyConnectors,
        home_switch: Position,
        home_row_index: usize,
    ) -> Self {
        let y_offset = -key_connectors.connector.length() - PAD_SIZE.y;
        let (offsets_below, offsets_above) = key_connectors.offsets.split_at(home_row_index);

        let mut position = home_switch;
        let switches_below = offsets_below
            .iter()
            .rev()
            .map(|&x_offset| {
                position -= position!(x_offset, y_offset, None);

                position
            })
            .collect();

        let mut position = home_switch;
        let switches_above = offsets_above
            .iter()
            .map(|&x_offset| {
                position += position!(x_offset, y_offset, None);

                position
            })
            .collect();

        Column {
            home_switch,
            switches_below,
            switches_above,
        }
    }

    /// Returns the positions of the switches in the column.
    pub fn positions(&self) -> impl Iterator<Item = &Position> {
        self.switches_below
            .iter()
            .rev()
            .chain(once(&self.home_switch))
            .chain(&self.switches_above)
    }

    /// Returns the position of the first switch in the column.
    pub fn first(&self) -> Position {
        self.switches_below
            .last()
            .copied()
            .unwrap_or(self.home_switch)
    }
}
