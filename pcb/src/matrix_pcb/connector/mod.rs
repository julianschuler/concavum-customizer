mod curved_connector;
mod straight_connector;

use curved_connector::CurvedConnector;
use model::matrix_pcb::{ClusterConnector, ColumnConnector, CONNECTOR_WIDTH, PAD_SIZE};
use straight_connector::StraightConnector;

use crate::{
    footprints::{
        ABOVE_ROW_PAD, BELOW_ROW_PAD, LEFT_OF_ROW_PAD, LOWER_COLUMN_PAD, ROW_PAD, UPPER_COLUMN_PAD,
    },
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{centered_track_offset, x_offset, AddPath, BOTTOM_LAYER, TOP_LAYER},
    path::Path,
    point,
    primitives::{Point, Position},
    unit::Length,
};

/// A connector between columns or clusters.
pub enum Connector {
    Straight(StraightConnector),
    Curved(CurvedConnector),
}

impl Connector {
    /// Creates a new connector from a column connector.
    pub fn from_column_connector(
        column_connector: &ColumnConnector,
        switch_position: Position,
    ) -> Self {
        match column_connector {
            ColumnConnector::Side(side_column_connector) => {
                Self::Straight(StraightConnector::from_side_column_connector(
                    side_column_connector,
                    switch_position,
                ))
            }
            ColumnConnector::Normal(normal_column_connector) => {
                Self::Curved(CurvedConnector::from_normal_column_connector(
                    normal_column_connector,
                    switch_position,
                ))
            }
        }
    }

    /// Creates a connector from a cluster connector and a start position.
    pub fn from_cluster_connector(
        cluster_connector: &ClusterConnector,
        start: Position,
        start_switch_position: Position,
    ) -> Self {
        Self::Curved(CurvedConnector::from_cluster_connector(
            cluster_connector,
            start,
            start_switch_position,
        ))
    }

    /// Returns the start position of the connector.
    pub fn start_position(&self) -> Position {
        match self {
            Connector::Straight(straight_connector) => straight_connector.start_position(),
            Connector::Curved(curved_connector) => curved_connector.start_position(),
        }
    }

    /// Returns the position of the switch at the start of the connector.
    pub fn start_switch_position(&self) -> Position {
        match self {
            Connector::Straight(straight_connector) => straight_connector.start_switch_position(),
            Connector::Curved(curved_connector) => curved_connector.start_switch_position(),
        }
    }

    /// Returns the attachment side of the start of the connector.
    pub fn start_attachment_side(&self) -> AttachmentSide {
        self.end_attachment_side().opposite()
    }

    /// Returns the end position of the connector.
    pub fn end_position(&self) -> Position {
        match self {
            Connector::Straight(straight_connector) => straight_connector.end_position(),
            Connector::Curved(curved_connector) => curved_connector.end_position(),
        }
    }

    /// Returns the position of the switch at the end of the connector.
    pub fn end_switch_position(&self) -> Position {
        match self {
            Connector::Straight(straight_connector) => straight_connector.end_switch_position(),
            Connector::Curved(curved_connector) => curved_connector.end_switch_position(),
        }
    }

    /// Returns the attachment side of the end of the connector.
    pub fn end_attachment_side(&self) -> AttachmentSide {
        match self {
            Connector::Straight(_) => AttachmentSide::Center,
            Connector::Curved(curved_connector) => curved_connector.end_attachment_side(),
        }
    }

    /// Adds the outline of the connector to the PCB.
    pub fn add_outline(&self, pcb: &mut KicadPcb) {
        match self {
            Connector::Straight(straight_connector) => straight_connector.add_outline(pcb),
            Connector::Curved(curved_connector) => curved_connector.add_outline(pcb),
        }
    }

    /// Adds a track to the PCB with the given offset to the center.
    pub fn add_track(&self, pcb: &mut KicadPcb, offset: Length, layer: &'static str, net: &Net) {
        match self {
            Connector::Straight(straight_connector) => {
                straight_connector.add_track(pcb, offset, layer, net);
            }
            Connector::Curved(curved_connector) => {
                curved_connector.add_track(pcb, offset, layer, net);
            }
        }
    }

    /// Adds the column track connecting the column to the left.
    pub fn add_left_column_track(&self, pcb: &mut KicadPcb, column_net: &Net) {
        let position = self.start_position();
        let track_points = [
            position.point(),
            position + point!(-Length::from(PAD_SIZE.x / 2.0) + LOWER_COLUMN_PAD.x(), 0),
        ];

        pcb.add_track(&track_points, TOP_LAYER, column_net);
    }

    /// Adds the column track connecting the column to the right.
    pub fn add_right_column_track(&self, pcb: &mut KicadPcb, track_count: usize, column_net: &Net) {
        let attachment_side = self.end_attachment_side();
        let start = point!(
            -PAD_SIZE.x / 2.0,
            attachment_side.y_offset() + centered_track_offset(0, track_count)
        );

        let end = point!(LOWER_COLUMN_PAD.x(), ROW_PAD.y());

        let track_path = match attachment_side {
            AttachmentSide::Top => Path::angled_start(start, UPPER_COLUMN_PAD),
            AttachmentSide::Center => {
                let center = point!(0, BELOW_ROW_PAD.y());

                Path::angled_end(start, center).join(&Path::angled_start(center, end))
            }
            AttachmentSide::Bottom => Path::angled_start(start, end),
        }
        .at(self.end_switch_position());

        pcb.add_track(&track_path, TOP_LAYER, column_net);
    }

    /// Adds the home row tracks connecting to the left and right columns to the PCB.
    pub fn add_home_row_tracks(&self, pcb: &mut KicadPcb, net: &Net, home_row_offset: Length) {
        for (left, sign_x, attachment_side, position) in [
            (
                true,
                1.0,
                self.start_attachment_side(),
                self.start_switch_position(),
            ),
            (
                false,
                -1.0,
                self.end_attachment_side(),
                self.end_switch_position(),
            ),
        ] {
            let y_offset = attachment_side.y_offset() - home_row_offset;
            let start_point = point!(sign_x * PAD_SIZE.x / 2.0, y_offset);

            let home_row_track_path = if matches!(attachment_side, AttachmentSide::Top) {
                let first_path_point = point!(2.4, -1.6);
                let second_path_point = point!(0.7, -5.6);

                Path::angled_start(start_point, second_path_point)
                    .join(&Path::angled_start(second_path_point, first_path_point))
                    .join(&Path::angled_start(first_path_point, ABOVE_ROW_PAD))
            } else {
                Path::angled_start(
                    start_point,
                    if left {
                        if matches!(attachment_side, AttachmentSide::Center) {
                            ABOVE_ROW_PAD
                        } else {
                            BELOW_ROW_PAD
                        }
                    } else {
                        LEFT_OF_ROW_PAD
                    },
                )
            }
            .append(ROW_PAD)
            .at(position);
            pcb.add_track(&home_row_track_path, BOTTOM_LAYER, net);
        }
    }

    /// Returns the track and its attachment point for the row directly above/below the home row.
    pub fn row_track_and_attachment_point(
        &self,
        other: &Self,
        left_connector_path: &Path,
        right_connector_path: &Path,
        track_offset: Length,
        above: bool,
    ) -> (Path, Point) {
        let position = self.end_switch_position();

        let left_attachment_point = *left_connector_path
            .first()
            .expect("connector track should always have points");
        let right_attachment_point = *right_connector_path
            .last()
            .expect("connector track should always have points");

        let (edge_side, sign, chamfer_depth) = if above {
            (AttachmentSide::Top, 1, Length::new(1.4))
        } else {
            (AttachmentSide::Bottom, -1, Length::new(3.0))
        };

        let left_is_at_edge = self.end_attachment_side() == edge_side;
        let right_is_at_edge = other.start_attachment_side() == edge_side;

        let y_offset = if above {
            (-6.6).into()
        } else {
            BELOW_ROW_PAD.y()
        };
        let x_offset = x_offset(0);

        let center_point = position + point!(0, y_offset);
        let left_chamfer_points = Path::new([
            point!(-x_offset, y_offset + sign * chamfer_depth),
            point!(-x_offset + chamfer_depth, y_offset),
        ])
        .at(position);
        let right_chamfer_points = Path::new([
            point!(x_offset - chamfer_depth, y_offset),
            point!(x_offset, y_offset + sign * chamfer_depth),
        ])
        .at(position);

        let path = if left_is_at_edge {
            if right_is_at_edge {
                Path::new([left_attachment_point, right_attachment_point])
            } else {
                Path::angled_start(left_attachment_point, center_point)
                    .join(&right_chamfer_points)
                    .join(right_connector_path)
            }
        } else if right_is_at_edge {
            left_connector_path
                .clone()
                .join(&left_chamfer_points)
                .join(&Path::angled_end(center_point, right_attachment_point))
        } else {
            left_connector_path
                .clone()
                .join(&left_chamfer_points)
                .join(&right_chamfer_points)
                .join(right_connector_path)
        };

        let attachment_point = if right_is_at_edge {
            point!(x_offset, edge_side.y_offset() + track_offset)
        } else {
            point!(x_offset - chamfer_depth, y_offset)
        };

        (path, attachment_point)
    }
}

/// The attachment side of the connector.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttachmentSide {
    Top,
    Center,
    Bottom,
}

impl AttachmentSide {
    /// Returns the Y offset corresponding to the attachment side.
    pub fn y_offset(self) -> Length {
        (match self {
            AttachmentSide::Top => -1.0,
            AttachmentSide::Center => 0.0,
            AttachmentSide::Bottom => 1.0,
        } * (PAD_SIZE.y - CONNECTOR_WIDTH)
            / 2.0)
            .into()
    }

    /// Returns the opposite attachment side.
    pub fn opposite(self) -> Self {
        match self {
            AttachmentSide::Top => AttachmentSide::Bottom,
            AttachmentSide::Center => AttachmentSide::Center,
            AttachmentSide::Bottom => AttachmentSide::Top,
        }
    }
}
