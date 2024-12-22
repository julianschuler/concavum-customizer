use model::matrix_pcb::{CONNECTOR_WIDTH, FFC_PAD_OFFSET, FFC_PAD_SIZE, PAD_SIZE};

use crate::{
    footprints::ROW_PAD,
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{
        centered_track_offset, features::Column, nets::Nets, AddPath, BOTTOM_LAYER, TOP_LAYER,
    },
    path::Path,
    point, position,
    primitives::Position,
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

    /// Adds the tracks for the FFC connector to the PCB.
    pub fn add_tracks(
        &self,
        pcb: &mut KicadPcb,
        nets: &Nets,
        row_count: usize,
        column_count: usize,
        thumb_switch_count: usize,
    ) {
        self.add_cluster_connector_tracks(pcb, &nets, thumb_switch_count);
    }

    /// Adds the tracks between the cluster connector and FFC connector
    fn add_cluster_connector_tracks(
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

        let row_path = Path::angled_center(
            point!(0, pad_bottom_offset),
            point!(Self::pad_x_offset(11), Self::Y_OFFSET + Self::PAD_OFFSET),
        )
        .at(self.anchor);
        pcb.add_track(&row_path, TOP_LAYER, row_net);

        let first_column_path = Path::angled_center(
            point!(
                centered_track_offset(0, thumb_switch_count),
                pad_bottom_offset
            ),
            point!(Self::pad_x_offset(0), Self::Y_OFFSET + Self::PAD_OFFSET),
        )
        .at(self.anchor);
        pcb.add_track(&first_column_path, BOTTOM_LAYER, first_column_net);

        for (i, column_net) in column_nets.iter().enumerate() {
            let path = Path::angled_start(
                point!(
                    centered_track_offset(i + 1, thumb_switch_count),
                    pad_bottom_offset
                ),
                point!(Self::pad_x_offset(i + 6), Self::Y_OFFSET + Self::PAD_OFFSET),
            )
            .at(self.anchor);
            pcb.add_track(&path, BOTTOM_LAYER, column_net);
        }
    }

    /// Returns the offset in the X direction of the connector pad with the given index.
    #[allow(clippy::cast_precision_loss)]
    fn pad_x_offset(index: usize) -> Length {
        (index as f32 - 5.5) * Self::PITCH
    }
}
