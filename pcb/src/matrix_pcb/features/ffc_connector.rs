use model::matrix_pcb::{CONNECTOR_WIDTH, FFC_PAD_OFFSET, FFC_PAD_SIZE, PAD_SIZE};

use crate::{
    footprints::{LOWER_COLUMN_PAD, ROW_PAD, UPPER_COLUMN_PAD},
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{
        centered_track_offset, features::Column, nets::Nets, track_offset, x_offset, AddPath,
        BOTTOM_LAYER, TOP_LAYER,
    },
    path::Path,
    point, position,
    primitives::{Point, Position},
    unit::{IntoAngle, Length},
};

/// An FFC connector.
pub struct FfcConnector {
    anchor: Position,
}

impl FfcConnector {
    const Y_OFFSET: Length = Length::new(5.5);
    const PITCH: Length = Length::new(1.0);
    const PAD_OFFSET: Length = Length::new(0.6);

    /// Creates a new FFC connector from the given anchor switch position.
    pub fn from_anchor(anchor: Position) -> Self {
        Self { anchor }
    }

    /// Calculates the start of the cluster connector.
    pub fn cluster_connector_start(&self) -> Position {
        self.anchor + position!(0, FFC_PAD_OFFSET + FFC_PAD_SIZE.y / 2.0, Some(-90.deg()))
    }

    /// Returns the position of the FFC connector.
    pub fn position(&self) -> Position {
        self.anchor + position!(0, Self::Y_OFFSET, None)
    }

    /// Adds the outline of the FFC connector to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        for sign in [-1.0, 1.0] {
            let pad_top_offset = FFC_PAD_OFFSET - FFC_PAD_SIZE.y / 2.0;
            let pad_bottom_offset = FFC_PAD_OFFSET + FFC_PAD_SIZE.y / 2.0;

            let outline_points = [
                self.anchor + point!(sign * CONNECTOR_WIDTH / 2.0, pad_bottom_offset),
                self.anchor + point!(sign * FFC_PAD_SIZE.x / 2.0, pad_bottom_offset),
                self.anchor + point!(sign * FFC_PAD_SIZE.x / 2.0, pad_top_offset),
                self.anchor + point!(sign * PAD_SIZE.x / 2.0, pad_top_offset),
                self.anchor + point!(sign * PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
            ];

            pcb.add_outline_path(&outline_points);
        }
    }

    /// Adds the tracks between the rows and FFC connector to the PCB.
    pub fn add_row_tracks(&self, pcb: &mut KicadPcb, row_nets: &[Net], ffc_column: &Column) {
        let (first_row_net, row_nets) = row_nets.split_first().expect("there is at least one row");
        let first_pad_x_offset = Self::pad_x_offset(5);
        let first_row_path = Path::angled_start(
            ROW_PAD,
            point!(first_pad_x_offset, Self::Y_OFFSET - Self::PAD_OFFSET),
        )
        .append(point!(first_pad_x_offset, Self::Y_OFFSET))
        .at(self.anchor);
        pcb.add_track(&first_row_path, BOTTOM_LAYER, first_row_net);

        for (i, (&position, net)) in ffc_column.positions().skip(1).zip(row_nets).enumerate() {
            let x_offset = -x_offset(i);
            let pad_x_offset = Self::pad_x_offset(4 - i);

            let path = Path::new([ROW_PAD])
                .join(&Path::angled_center(
                    point!(0, ROW_PAD.y()),
                    point!(x_offset, PAD_SIZE.y / 2.0),
                ))
                .at(position)
                .join(
                    &Path::angled_start(
                        point!(x_offset, -PAD_SIZE.y / 2.0),
                        point!(pad_x_offset, Self::Y_OFFSET - Self::PAD_OFFSET),
                    )
                    .append(point!(pad_x_offset, Self::Y_OFFSET))
                    .at(self.anchor),
                );

            pcb.add_track(&path, TOP_LAYER, net);
        }
    }

    /// Adds the tracks between the columns and FFC connector to the PCB.
    pub fn add_column_tracks(
        &self,
        pcb: &mut KicadPcb,
        nets: &Nets,
        row_count: usize,
        column_count: usize,
        left_column_connector: Option<Position>,
        right_column_connector: Option<Position>,
    ) {
        let (first_column_net, column_nets) = nets.columns[..column_count]
            .split_first()
            .expect("there is at least one column");

        if let Some(left_column_connector) = left_column_connector {
            self.add_left_column_track(pcb, left_column_connector, first_column_net, row_count);

            let (second_column_net, column_nets) = column_nets
                .split_first()
                .expect("there is at least one other column");
            self.add_center_column_track(pcb, second_column_net, false);

            if let Some(right_column_connector) = right_column_connector {
                self.add_right_column_tracks(pcb, right_column_connector, column_nets, false);
            }
        } else {
            self.add_center_column_track(pcb, first_column_net, true);

            if let Some(right_column_connector) = right_column_connector {
                self.add_right_column_tracks(pcb, right_column_connector, column_nets, true);
            }
        };
    }

    /// Adds the tracks between the cluster and FFC connector.
    pub fn add_cluster_connector_tracks(
        &self,
        pcb: &mut KicadPcb,
        nets: &Nets,
        thumb_switch_count: usize,
    ) {
        let pad_bottom_offset = FFC_PAD_OFFSET + FFC_PAD_SIZE.y / 2.0;
        let row_net = &nets.rows[0];
        let (first_column_net, column_nets) = nets.columns[..thumb_switch_count]
            .split_first()
            .expect("there is at least one thumb switch");

        let row_pad_x_offset = Self::pad_x_offset(11);
        let row_path = Path::angled_center(
            point!(centered_track_offset(1, 2), pad_bottom_offset),
            point!(row_pad_x_offset, Self::Y_OFFSET + Self::PAD_OFFSET),
        )
        .append(point!(row_pad_x_offset, Self::Y_OFFSET))
        .at(self.anchor);
        pcb.add_track(&row_path, TOP_LAYER, row_net);

        let first_pad_x_offset = Self::pad_x_offset(0);
        let first_column_path = Path::angled_center(
            point!(centered_track_offset(0, 2), pad_bottom_offset),
            point!(first_pad_x_offset, Self::Y_OFFSET + Self::PAD_OFFSET),
        )
        .append(point!(first_pad_x_offset, Self::Y_OFFSET))
        .at(self.anchor);
        pcb.add_track(&first_column_path, TOP_LAYER, first_column_net);

        for (i, column_net) in column_nets.iter().enumerate() {
            let pad_x_offset = Self::pad_x_offset(i + 6);
            let path = angled_path(
                point!(
                    centered_track_offset(i, thumb_switch_count - 1),
                    pad_bottom_offset
                ),
                point!(pad_x_offset, Self::Y_OFFSET + Self::PAD_OFFSET),
            )
            .append(point!(pad_x_offset, Self::Y_OFFSET))
            .at(self.anchor);
            pcb.add_track(&path, BOTTOM_LAYER, column_net);
        }
    }

    /// Adds the track connecting the left column and FFC connector.
    fn add_left_column_track(
        &self,
        pcb: &mut KicadPcb,
        column_connector: Position,
        net: &Net,
        row_count: usize,
    ) {
        let x_offset = -x_offset(row_count - 1);
        let connector_offset = Length::from(PAD_SIZE.x / 2.0) + x_offset;
        let pad_x_offset = Self::pad_x_offset(0);

        let track_path = Path::chamfered(
            point!(0, 0),
            point!(connector_offset, connector_offset),
            connector_offset,
            false,
        )
        .at(column_connector)
        .join(
            &Path::angled_start(
                point!(x_offset, -PAD_SIZE.y / 2.0),
                point!(pad_x_offset, Self::Y_OFFSET - Self::PAD_OFFSET),
            )
            .append(point!(pad_x_offset, Self::Y_OFFSET))
            .at(self.anchor),
        );
        pcb.add_track(&track_path, TOP_LAYER, net);
    }

    /// Adds the track connecting the center column and FFC connector.
    fn add_center_column_track(&self, pcb: &mut KicadPcb, net: &Net, is_first_column: bool) {
        if is_first_column {
            let track_path = Path::chamfered(
                UPPER_COLUMN_PAD,
                point!(Self::pad_x_offset(0), Self::Y_OFFSET),
                6.6.into(),
                true,
            )
            .at(self.anchor);

            pcb.add_track(&track_path, BOTTOM_LAYER, net);
        } else {
            let x_offset = Self::pad_x_offset(6);
            let track_path = Path::angled_start(LOWER_COLUMN_PAD, point!(x_offset, 3))
                .append(point!(x_offset, Self::Y_OFFSET))
                .at(self.anchor);

            pcb.add_track(&track_path, TOP_LAYER, net);
        }
    }

    /// Adds the tracks connecting the right columns and FFC connector.
    fn add_right_column_tracks(
        &self,
        pcb: &mut KicadPcb,
        right_column_connector: Position,
        column_nets: &[Net],
        contains_second_column: bool,
    ) {
        for (i, net) in column_nets.iter().enumerate() {
            let pad_x_offset = Self::pad_x_offset(i + 6 + usize::from(!contains_second_column));
            let pad_y_offset = if contains_second_column && i == 0 {
                3.into()
            } else {
                Self::Y_OFFSET - Self::PAD_OFFSET
            };
            let connector_offset = Length::from(PAD_SIZE.x / 2.0) - x_offset(0);

            let path = Path::chamfered(
                point!(0, centered_track_offset(0, column_nets.len())),
                point!(-connector_offset, connector_offset),
                0.8.into(),
                true,
            )
            .offset(track_offset(i))
            .at(right_column_connector)
            .join(
                &Path::angled_start(
                    point!(x_offset(i), -PAD_SIZE.y / 2.0),
                    point!(pad_x_offset, pad_y_offset),
                )
                .append(point!(pad_x_offset, Self::Y_OFFSET))
                .at(self.anchor),
            );

            pcb.add_track(&path, TOP_LAYER, net);
        }
    }

    /// Returns the offset in the X direction of the connector pad with the given index.
    #[allow(clippy::cast_precision_loss)]
    fn pad_x_offset(index: usize) -> Length {
        (index as f32 - 5.5) * Self::PITCH
    }
}

/// Creates a new angled path with the non-angled section at the start if it is vertical or in
/// the center if it is horizontal.
fn angled_path(start: Point, end: Point) -> Path {
    let difference = end - start;

    if difference.y.abs() >= difference.x.abs() {
        Path::angled_start(start, end)
    } else {
        Path::angled_center(start, end)
    }
}
