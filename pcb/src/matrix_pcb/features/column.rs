use std::iter::once;

use model::matrix_pcb::{
    ColumnKeyConnectors, Segment, CONNECTOR_WIDTH, PAD_SIZE, ROUTER_BIT_DIAMETER,
};

use crate::{
    footprints::{
        ABOVE_ROW_PAD, BELOW_ROW_PAD, LEFT_OF_ROW_PAD, LOWER_COLUMN_PAD, ROW_PAD, UPPER_COLUMN_PAD,
    },
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{
        centered_track_offset,
        connector::{AttachmentSide, Connector},
        nets::Nets,
        track_offset, x_offset, AddPath, BOTTOM_LAYER, TOP_LAYER, TRACK_CLEARANCE, TRACK_WIDTH,
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

    /// Adds the tracks connecting the rows in an inner column to the PCB.
    pub fn add_inner_column_row_tracks(
        &self,
        pcb: &mut KicadPcb,
        nets: &Nets,
        left_column_connector: &Connector,
        right_column_connector: &Connector,
    ) {
        const CONNECTOR_PATH_HEIGHT: f32 = 1.4;

        let row_count = nets.finger_rows().len();
        let connector_offset = Length::from(PAD_SIZE.x / 2.0) - x_offset(0);

        for (switches, nets, above, sign) in [
            (&self.switches_below, nets.lower_finger_rows(), false, -1f32),
            (&self.switches_above, nets.upper_finger_rows(), true, 1f32),
        ] {
            let row_pad_attachment_point = if above { BELOW_ROW_PAD } else { ABOVE_ROW_PAD };

            if let Some(&first) = switches.first() {
                let connector_track_offset =
                    sign * centered_track_offset(switches.len() - 1, row_count);

                let left_column_connector_path = Path::chamfered(
                    point!(0, connector_track_offset),
                    point!(connector_offset, -sign * CONNECTOR_PATH_HEIGHT),
                    0.8.into(),
                    above,
                )
                .at(left_column_connector.end_position());
                let right_column_connector_path = Path::chamfered(
                    point!(-connector_offset, -sign * CONNECTOR_PATH_HEIGHT),
                    point!(0, connector_track_offset),
                    0.8.into(),
                    above,
                )
                .at(right_column_connector.start_position());

                for (i, (window, net)) in once([self.home_switch, first].as_slice())
                    .chain(switches.windows(2))
                    .zip(nets)
                    .enumerate()
                {
                    let first_switch = window[0];
                    let second_switch = window[1];

                    let (column_connector_path, track_attachment_point) = if i > 0 {
                        let offset = sign * track_offset(i);
                        let double_chamfer = double_chamfer(i, above);

                        let column_connector_path = left_column_connector_path
                            .offset(offset)
                            .join(&double_chamfer.at(first_switch))
                            .join(&right_column_connector_path.offset(offset));

                        (column_connector_path, double_chamfer[2])
                    } else {
                        left_column_connector.row_track_and_attachment_point(
                            right_column_connector,
                            &left_column_connector_path,
                            &right_column_connector_path,
                            connector_track_offset,
                            above,
                        )
                    };
                    pcb.add_track(&column_connector_path, BOTTOM_LAYER, net);

                    let row_path = Path::angled_end(
                        track_attachment_point,
                        point!(x_offset(0), -f64::from(sign) * PAD_SIZE.y / 2.0),
                    )
                    .at(first_switch)
                    .join(
                        &Path::angled_start(
                            point!(x_offset(0), f64::from(sign) * PAD_SIZE.y / 2.0),
                            row_pad_attachment_point,
                        )
                        .append(ROW_PAD)
                        .at(second_switch),
                    );
                    pcb.add_track(&row_path, BOTTOM_LAYER, net);
                }
            }
        }
    }

    /// Adds the tracks connecting the rows in a left- or rightmost column to the PCB.
    pub fn add_outer_column_row_tracks(
        &self,
        pcb: &mut KicadPcb,
        nets: &Nets,
        attachment_side: AttachmentSide,
        left: bool,
    ) {
        let sign_x: f64 = if left { 1.0 } else { -1.0 };
        let track_x_offset = sign_x * x_offset(0);
        let row_count = nets.finger_rows().len();

        let (y_offset_below, y_offset_above) = if matches!(attachment_side, AttachmentSide::Center)
        {
            (ROW_PAD.y(), UPPER_COLUMN_PAD.y())
        } else {
            (
                (PAD_SIZE.y / 2.0 + 2.0).into(),
                (-PAD_SIZE.y / 2.0 - 2.0).into(),
            )
        };
        let (offsets_below, offsets_above) = self.offsets.split_at(self.switches_below.len());
        let offsets_below: Vec<_> = offsets_below.iter().rev().map(|&offset| -offset).collect();

        for (switches, offsets, row_nets, y_offset, above, sign_y) in [
            (
                &self.switches_below,
                offsets_below.as_slice(),
                nets.lower_finger_rows(),
                y_offset_below,
                false,
                -1.0,
            ),
            (
                &self.switches_above,
                offsets_above,
                nets.upper_finger_rows(),
                y_offset_above,
                true,
                1.0,
            ),
        ] {
            if !switches.is_empty() {
                let start_path = Path::chamfered(
                    point!(
                        sign_x * PAD_SIZE.x / 2.0,
                        attachment_side.y_offset()
                            + sign_y * centered_track_offset(switches.len() - 1, row_count)
                    ),
                    point!(track_x_offset, y_offset + sign_y.into()),
                    0.8.into(),
                    left != above,
                )
                .at(self.home_switch);

                for (i, net) in row_nets.iter().enumerate() {
                    let path = once(&self.home_switch)
                        .chain(switches)
                        .zip(offsets)
                        .take(i + 1)
                        .map(|(&switch, &offset)| {
                            Path::angled_end_center(
                                point!(track_x_offset, y_offset),
                                point!(track_x_offset + offset, sign_y * -PAD_SIZE.y / 2.0),
                            )
                            .append(point!(
                                track_x_offset + offset,
                                sign_y * (-PAD_SIZE.y / 2.0 - 1.0)
                            ))
                            .at(switch)
                        })
                        .fold(start_path.clone(), |path, other| path.join(&other))
                        .offset(sign_x * sign_y * -track_offset(i))
                        .join(&row_path(i, left, above).at(switches[i]));

                    pcb.add_track(&path, BOTTOM_LAYER, net);
                }
            }
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

/// Creates a double chamfer for the given index and side.
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

/// Returns the row path with the given index, going to the respective sides.
fn row_path(index: usize, left: bool, above: bool) -> Path {
    if left {
        if above {
            Path::angled_start_center(point!(x_offset(index), PAD_SIZE.y / 2.0), BELOW_ROW_PAD)
        } else {
            Path::angled_start(point!(x_offset(index), -PAD_SIZE.y / 2.0), ABOVE_ROW_PAD)
        }
    } else if above {
        Path::angled_center(point!(-x_offset(index), PAD_SIZE.y / 2.0), LEFT_OF_ROW_PAD)
    } else {
        Path::angled_start(point!(-x_offset(index), -PAD_SIZE.y / 2.0), LEFT_OF_ROW_PAD)
    }
    .append(ROW_PAD)
}
