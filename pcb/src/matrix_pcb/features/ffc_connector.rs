use model::matrix_pcb::{CONNECTOR_WIDTH, FFC_PAD_OFFSET, FFC_PAD_SIZE, PAD_SIZE};

use crate::{
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{
        centered_track_offset,
        nets::{self, Nets},
        AddPath, BOTTOM_LAYER, TOP_LAYER,
    },
    path::Path,
    point, position,
    primitives::{Point, Position},
    unit::IntoUnit,
};

/// An FFC connector.
pub struct FfcConnector {
    anchor: Position,
}

impl FfcConnector {
    const OFFSET: f32 = 5.5;
    const PITCH: f32 = 1.0;

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
        self.anchor + position!(0, Self::OFFSET, None)
    }

    /// Adds the outline of the FFC connector to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        for sign in [-1.0, 1.0] {
            let ffc_pad_top_offset = FFC_PAD_OFFSET - FFC_PAD_SIZE.y / 2.0;
            let ffc_pad_bottom_offset = FFC_PAD_OFFSET + FFC_PAD_SIZE.y / 2.0;

            let outline_points = [
                self.anchor + point!(sign * CONNECTOR_WIDTH / 2.0, ffc_pad_bottom_offset),
                self.anchor + point!(sign * FFC_PAD_SIZE.x / 2.0, ffc_pad_bottom_offset),
                self.anchor + point!(sign * FFC_PAD_SIZE.x / 2.0, ffc_pad_top_offset),
                self.anchor + point!(sign * PAD_SIZE.x / 2.0, ffc_pad_top_offset),
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
        const PAD_OFFSET: f32 = 0.6;

        let ffc_pad_bottom_offset = FFC_PAD_OFFSET + FFC_PAD_SIZE.y / 2.0;
        let row_net = &nets.rows[0];
        let (first_column_net, column_nets) = nets.columns[..thumb_switch_count]
            .split_first()
            .expect("there is at least one thumb switch");

        let row_path = Path::angled_center(
            point!(0, ffc_pad_bottom_offset),
            point!(5.5 * Self::PITCH, Self::OFFSET + PAD_OFFSET),
        )
        .at(self.anchor);
        pcb.add_track(&row_path, TOP_LAYER, row_net);

        let first_column_path = Path::angled_center(
            Point::new(
                centered_track_offset(0, thumb_switch_count),
                ffc_pad_bottom_offset.mm(),
            ),
            point!(-5.5 * Self::PITCH, Self::OFFSET + PAD_OFFSET),
        )
        .at(self.anchor);
        pcb.add_track(&first_column_path, BOTTOM_LAYER, first_column_net);

        for (i, column_net) in column_nets.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let ffc_connector_x_offset = (i as f32 + 0.5) * Self::PITCH;
            let path = Path::angled_start(
                Point::new(
                    centered_track_offset(i + 1, thumb_switch_count),
                    ffc_pad_bottom_offset.mm(),
                ),
                point!(ffc_connector_x_offset, Self::OFFSET + PAD_OFFSET),
            )
            .at(self.anchor);
            pcb.add_track(&path, BOTTOM_LAYER, column_net);
        }
    }
}
