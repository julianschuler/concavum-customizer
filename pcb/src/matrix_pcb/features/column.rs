use std::iter::once;

use model::matrix_pcb::{
    ColumnKeyConnectors, Segment, CONNECTOR_WIDTH, PAD_SIZE, ROUTER_BIT_DIAMETER,
};

use crate::{
    footprints::{ABOVE_ROW_PAD, BELOW_ROW_PAD, LOWER_COLUMN_PAD, ROW_PAD, UPPER_COLUMN_PAD},
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{
        centered_track_offset, nets::Nets, track_offset, x_offset, AddPath, TOP_LAYER, 
        TRACK_CLEARANCE, TRACK_WIDTH,
    },
    path::Path,
    point,
    primitives::Position,
    unit::Length,
};

/// The positions of a column of finger key switches.
pub struct Column {
    home_switch: Position,
    switches_below: Vec<Position>,
    switches_above: Vec<Position>,
    offsets: Vec<Length>,
}

impl Column {
    /// Creates a new column from the corresponding key connectors and home switch position.
    pub fn from_key_connectors(
        key_connectors: &ColumnKeyConnectors,
        home_switch: Position,
        home_row_index: usize,
    ) -> Self {
        let y_offset = (-key_connectors.connector.length() - PAD_SIZE.y).into();
        let offsets: Vec<_> = key_connectors
            .offsets
            .iter()
            .copied()
            .map(Into::into)
            .collect();
        let (offsets_below, offsets_above) = offsets.split_at(home_row_index);

        let mut position = home_switch;
        let switches_below = offsets_below
            .iter()
            .rev()
            .map(|&x_offset| {
                position -= Position::new(x_offset, y_offset, None);

                position
            })
            .collect();

        let mut position = home_switch;
        let switches_above = offsets_above
            .iter()
            .map(|&x_offset| {
                position += Position::new(x_offset, y_offset, None);

                position
            })
            .collect();

        Column {
            home_switch,
            switches_below,
            switches_above,
            offsets,
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

    /// Returns the position of the last switch in the column.
    pub fn last(&self) -> Position {
        self.switches_above
            .last()
            .copied()
            .unwrap_or(self.home_switch)
    }

    /// Adds the outline of the column switches to the PCB.
    pub fn add_outline(
        &self,
        pcb: &mut KicadPcb,
        left_connector_position: Option<Position>,
        right_connector_position: Option<Position>,
        is_ffc_column: bool,
    ) {
        for ((&bottom_switch, &top_switch), &offset) in self
            .positions()
            .zip(self.positions().skip(1))
            .zip(&self.offsets)
        {
            add_connector_outline(pcb, bottom_switch, top_switch, offset);
        }

        if let Some((&last, remaining)) = self.switches_below.split_last() {
            if !is_ffc_column {
                add_bottom_switch_outline(pcb, last);
            }

            for &position in remaining {
                add_pad_outline(pcb, position);
            }
        }
        for &position in &self.switches_above {
            add_pad_outline(pcb, position);
        }

        let last_position = self.last();
        let top_outline = [
            last_position + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
            last_position + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
        ];
        pcb.add_outline_path(&top_outline);

        self.add_home_switch_outline(pcb, left_connector_position, right_connector_position);
    }

    /// Adds the track connecting the switches in the column to each other.
    pub fn add_switch_tracks(&self, pcb: &mut KicadPcb, column_net: &Net) {
        let x_offset = x_offset(0) - TRACK_WIDTH - TRACK_CLEARANCE;

        for ((&bottom_switch, &top_switch), &offset) in self
            .positions()
            .zip(self.positions().skip(1))
            .zip(&self.offsets)
        {
            let negative_offset = offset.min(0.into());
            let positive_offset = offset.max(0.into());

            let track_path = Path::angled_end_center(
                UPPER_COLUMN_PAD,
                point!(x_offset + negative_offset, -PAD_SIZE.y / 2.0),
            )
            .at(bottom_switch)
            .join(
                &Path::angled_start_center(
                    point!(x_offset - positive_offset, PAD_SIZE.y / 2.0),
                    point!(LOWER_COLUMN_PAD.x(), ROW_PAD.y()),
                )
                .append(LOWER_COLUMN_PAD)
                .at(top_switch),
            );

            pcb.add_track(&track_path, TOP_LAYER, column_net);
        }
    }

    /// Adds the tracks connecting the columns.
    pub fn add_column_tracks(
        &self,
        pcb: &mut KicadPcb,
        column_nets: &[Net],
        left_column_connector: Position,
        right_column_connector: Position,
    ) {
        let x_offset = x_offset(0);
        let connector_offset = Length::from(PAD_SIZE.x / 2.0) - x_offset;

        let path = Path::chamfered(
            point!(0, centered_track_offset(1, column_nets.len() + 1)),
            point!(connector_offset, PAD_SIZE.y / 2.0),
            1.into(),
            false,
        )
        .at(left_column_connector)
        .join(&double_chamfer(0, false).at(self.first()))
        .join(
            &Path::chamfered(
                point!(-connector_offset, PAD_SIZE.y / 2.0),
                point!(0, centered_track_offset(0, column_nets.len())),
                1.into(),
                false,
            )
            .at(right_column_connector),
        );

        for (i, column_net) in column_nets.iter().enumerate() {
            pcb.add_track(&path.offset(-track_offset(i)), TOP_LAYER, column_net);
        }
    }

    /// Adds the outline of the home switch to the PCB.
    fn add_home_switch_outline(
        &self,
        pcb: &mut KicadPcb,
        left_connector_position: Option<Position>,
        right_connector_position: Option<Position>,
    ) {
        for (position, x_offset) in [
            (left_connector_position, -PAD_SIZE.x / 2.0),
            (right_connector_position, PAD_SIZE.x / 2.0),
        ] {
            if let Some(position) = position {
                let lower_outline_points = [
                    self.home_switch + point!(x_offset, PAD_SIZE.y / 2.0),
                    position + point!(0, CONNECTOR_WIDTH / 2.0),
                ];
                let upper_outline_points = [
                    position + point!(0, -CONNECTOR_WIDTH / 2.0),
                    self.home_switch + point!(x_offset, -PAD_SIZE.y / 2.0),
                ];
                pcb.add_outline_path(&lower_outline_points);
                pcb.add_outline_path(&upper_outline_points);
            } else {
                let outline_points = [
                    self.home_switch + point!(x_offset, PAD_SIZE.y / 2.0),
                    self.home_switch + point!(x_offset, -PAD_SIZE.y / 2.0),
                ];
                pcb.add_outline_path(&outline_points);
            }
        }
    }
}

/// Adds the outline of the connectors within a column to the PCB.
fn add_connector_outline(
    pcb: &mut KicadPcb,
    bottom_switch: Position,
    top_switch: Position,
    offset: Length,
) {
    let offset = f64::from(offset);
    let negative_offset = offset.min(0.0);
    let positive_offset = offset.max(0.0);

    let left_outline_points = [
        bottom_switch + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
        bottom_switch + point!(-PAD_SIZE.x / 2.0 + positive_offset, -PAD_SIZE.y / 2.0),
        top_switch + point!(-PAD_SIZE.x / 2.0 - negative_offset, PAD_SIZE.y / 2.0),
        top_switch + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
    ];
    let right_outline_points = [
        bottom_switch + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
        bottom_switch + point!(PAD_SIZE.x / 2.0 + negative_offset, -PAD_SIZE.y / 2.0),
        top_switch + point!(PAD_SIZE.x / 2.0 - positive_offset, PAD_SIZE.y / 2.0),
        top_switch + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
    ];
    pcb.add_outline_path(&left_outline_points);
    pcb.add_outline_path(&right_outline_points);

    if offset.abs() < PAD_SIZE.x - 2.0 * CONNECTOR_WIDTH - ROUTER_BIT_DIAMETER {
        let cutout_points = [
            bottom_switch
                + point!(
                    -PAD_SIZE.x / 2.0 + CONNECTOR_WIDTH + positive_offset,
                    -PAD_SIZE.y / 2.0
                ),
            bottom_switch
                + point!(
                    PAD_SIZE.x / 2.0 - CONNECTOR_WIDTH + negative_offset,
                    -PAD_SIZE.y / 2.0
                ),
            top_switch
                + point!(
                    PAD_SIZE.x / 2.0 - CONNECTOR_WIDTH - positive_offset,
                    PAD_SIZE.y / 2.0
                ),
            top_switch
                + point!(
                    -PAD_SIZE.x / 2.0 + CONNECTOR_WIDTH - negative_offset,
                    PAD_SIZE.y / 2.0
                ),
        ];
        pcb.add_outline_polygon(&cutout_points);
    }
}

/// Adds the outline of the bottom most switch to the given position to the PCB.
fn add_bottom_switch_outline(pcb: &mut KicadPcb, position: Position) {
    let outline_points = [
        position + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
        position + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
        position + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
        position + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
    ];

    pcb.add_outline_path(&outline_points);
}

/// Adds the outline of a single pad at the given position to the PCB.
fn add_pad_outline(pcb: &mut KicadPcb, position: Position) {
    for x_offset in [-PAD_SIZE.x / 2.0, PAD_SIZE.x / 2.0] {
        let outline_points = [
            position + point!(x_offset, PAD_SIZE.y / 2.0),
            position + point!(x_offset, -PAD_SIZE.y / 2.0),
        ];

        pcb.add_outline_path(&outline_points);
    }
}

/// Creates a double chamfer for the given the given index and side.
fn double_chamfer(index: usize, above: bool) -> Path {
    const CHAMFER_DEPTH: Length = Length::new(3.0);

    let x_offset = x_offset(index);
    let y_offset = if above { ABOVE_ROW_PAD } else { BELOW_ROW_PAD }.y();
    let sign = if above { 1 } else { -1 };

    Path::new([
        point!(-x_offset, y_offset + sign * CHAMFER_DEPTH),
        point!(-x_offset + CHAMFER_DEPTH, y_offset),
        point!(x_offset - CHAMFER_DEPTH, y_offset),
        point!(x_offset, y_offset + sign * CHAMFER_DEPTH),
    ])
}
