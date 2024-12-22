use model::matrix_pcb::{Segment, ThumbKeyConnectors, CONNECTOR_WIDTH, PAD_SIZE};

use crate::{
    footprints::{ROW_PAD, UPPER_COLUMN_PAD},
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{
        centered_track_offset, nets::Nets, track_offset, AddPath, BOTTOM_LAYER, TOP_LAYER,
        TRACK_CLEARANCE, TRACK_WIDTH,
    },
    path::Path,
    point, position,
    primitives::Position,
    unit::Length,
};

/// The positions of the thumb key switches.
pub struct ThumbSwitches(Vec<Position>);

impl ThumbSwitches {
    /// Creates a new set of thumb keys from the corresponding key connectors and position of the first one.
    pub fn from_key_connectors(
        key_connectors: &ThumbKeyConnectors,
        first_switch: Position,
    ) -> Self {
        let switch_distance = key_connectors.connector.length() + PAD_SIZE.x;

        #[allow(clippy::cast_precision_loss)]
        let positions = (0..=key_connectors.positions.len())
            .map(|i| first_switch + position!(i as f64 * switch_distance, 0, None))
            .collect();

        Self(positions)
    }

    /// Returns the positions of the thumb keys.
    pub fn positions(&self) -> &[Position] {
        &self.0
    }

    /// Adds the outline for the thumb switches to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        let first = {
            let this = &self;
            this.0
                .first()
                .copied()
                .expect("there is always at least one thumb key")
        };
        let last = {
            let this = &self;
            this.0
                .last()
                .copied()
                .expect("there is always at least one thumb key")
        };

        let outline_points = [
            first + point!(PAD_SIZE.x / 2.0 - CONNECTOR_WIDTH, -PAD_SIZE.y / 2.0),
            first + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
            first + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
            last + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0),
            last + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
            first + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0),
        ];
        pcb.add_outline_path(&outline_points);

        for window in self.0.windows(2) {
            let position = window[0];
            let next_position = window[1];

            let cutout_points = [
                position + point!(PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0 - CONNECTOR_WIDTH),
                position + point!(PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0 + CONNECTOR_WIDTH),
                next_position + point!(-PAD_SIZE.x / 2.0, -PAD_SIZE.y / 2.0 + CONNECTOR_WIDTH),
                next_position + point!(-PAD_SIZE.x / 2.0, PAD_SIZE.y / 2.0 - CONNECTOR_WIDTH),
            ];
            pcb.add_outline_polygon(&cutout_points);
        }
    }

    /// Adds the tracks for the thumb switches to the PCB.
    pub fn add_tracks(&self, pcb: &mut KicadPcb, nets: &Nets) {
        self.add_row_track(pcb, &nets.rows[0]);
        self.add_column_tracks(pcb, &nets.columns);
    }

    /// Adds the track connecting the row of the thumb switches.
    fn add_row_track(&self, pcb: &mut KicadPcb, row_net: &Net) {
        const X_OFFSET: Length = Length::new((PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0);
        const Y_OFFSET: Length = Length::new((PAD_SIZE.y - CONNECTOR_WIDTH) / 2.0);

        let (&first, rest) = self
            .0
            .split_first()
            .expect("there is always at least one thumb switch");

        if let Some((&last, rest)) = rest.split_last() {
            let chamfer_depth = Y_OFFSET - ROW_PAD.y() - Length::new(1.0);

            let second_segment = &Path::chamfered(
                point!(-PAD_SIZE.x / 2.0, Y_OFFSET),
                ROW_PAD,
                chamfer_depth,
                true,
            )
            .at(last);
            let path = Path::chamfered(
                ROW_PAD,
                point!(PAD_SIZE.x / 2.0, Y_OFFSET),
                chamfer_depth,
                true,
            )
            .at(first)
            .join(second_segment);
            pcb.add_track(&path, TOP_LAYER, row_net);

            let track_points = [
                first + point!(X_OFFSET, Y_OFFSET),
                first + point!(X_OFFSET, -PAD_SIZE.y / 2.0),
            ];
            pcb.add_track(&track_points, TOP_LAYER, row_net);

            for position in rest {
                let track_points = [
                    *position + ROW_PAD,
                    *position + point!(ROW_PAD.x(), Y_OFFSET),
                ];
                pcb.add_track(&track_points, TOP_LAYER, row_net);
            }
        } else {
            let path = Path::chamfered(
                point!(X_OFFSET, -PAD_SIZE.y / 2.0),
                point!(ROW_PAD.x(), ROW_PAD.y() + 1.into()),
                3.into(),
                false,
            )
            .append(ROW_PAD)
            .at(first);

            pcb.add_track(&path, TOP_LAYER, row_net);
        };
    }

    /// Adds the tracks connecting the columns of the thumb switches.
    fn add_column_tracks(&self, pcb: &mut KicadPcb, columns: &[Net]) {
        #[allow(clippy::approx_constant)]
        const TRACK_OFFSET: Length = Length::new(-6.28);
        const X_OFFSET: Length = Length::new((PAD_SIZE.x - CONNECTOR_WIDTH) / 2.0);

        let thumb_switch_count = self.positions().len();

        let (&first, rest) = self
            .0
            .split_first()
            .expect("there is always at least one thumb switch");
        let first_column_offset = centered_track_offset(0, thumb_switch_count);

        let path_center = point!(0, TRACK_OFFSET - TRACK_WIDTH / 2 - TRACK_CLEARANCE);
        let first_column_path = Path::chamfered(
            point!(first_column_offset + X_OFFSET, -PAD_SIZE.y / 2.0),
            path_center,
            1.into(),
            false,
        )
        .join(&Path::angled_start(path_center, UPPER_COLUMN_PAD))
        .at(first);
        pcb.add_track(&first_column_path, BOTTOM_LAYER, &columns[0]);

        let y_offset = TRACK_OFFSET + TRACK_WIDTH / 2;
        let start_offset = centered_track_offset(1, thumb_switch_count);
        let first_path_segment = Path::chamfered(
            point!(start_offset + X_OFFSET, -PAD_SIZE.y / 2.0),
            point!(PAD_SIZE.x / 2.0, y_offset),
            0.8.into(),
            true,
        )
        .at(first);

        for (i, &switch) in rest.iter().enumerate() {
            let offset = track_offset(i);
            let offset_path = first_path_segment.offset(offset).join(
                &Path::angled_start(
                    point!(-PAD_SIZE.x / 2.0, y_offset - offset),
                    UPPER_COLUMN_PAD,
                )
                .at(switch),
            );
            pcb.add_track(&offset_path, BOTTOM_LAYER, &columns[i + 1]);
        }
    }
}
