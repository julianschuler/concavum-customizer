mod curved_connector;
mod straight_connector;

use curved_connector::CurvedConnector;
use model::matrix_pcb::{ClusterConnector, ColumnConnector, CONNECTOR_WIDTH, PAD_SIZE};
use straight_connector::StraightConnector;

use crate::{
    footprints::{BELOW_ROW_PAD, LOWER_COLUMN_PAD, ROW_PAD, UPPER_COLUMN_PAD},
    kicad_pcb::{KicadPcb, Net},
    matrix_pcb::{centered_track_offset, AddPath, TOP_LAYER},
    path::Path,
    point,
    primitives::Position,
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
    pub fn from_cluster_connector(cluster_connector: &ClusterConnector, start: Position) -> Self {
        Self::Curved(CurvedConnector::from_cluster_connector(
            cluster_connector,
            start,
        ))
    }

    /// Returns the start position of the connector
    pub fn start_position(&self) -> Position {
        match self {
            Connector::Straight(straight_connector) => straight_connector.start_position(),
            Connector::Curved(curved_connector) => curved_connector.start_position(),
        }
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
            position - point!(Length::from(PAD_SIZE.x / 2.0) - LOWER_COLUMN_PAD.x(), 0),
        ];

        pcb.add_track(&track_points, TOP_LAYER, column_net);
    }

    /// Adds the column track connecting the column to the left.
    pub fn add_right_column_track(&self, pcb: &mut KicadPcb, track_count: usize, column_net: &Net) {
        let attachment_side = match self {
            Connector::Straight(_) => AttachmentSide::Center,
            Connector::Curved(curved_connector) => curved_connector.end_attachment_side(),
        };
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
}

/// The attachment side of the connector.
#[derive(Clone, Copy)]
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
}
