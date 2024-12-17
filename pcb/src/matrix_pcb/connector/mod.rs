mod curved_connector;
mod straight_connector;

use curved_connector::CurvedConnector;
use model::matrix_pcb::{ClusterConnector, ColumnConnector};
use straight_connector::StraightConnector;

use crate::{
    kicad_pcb::{KicadPcb, Net},
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

    /// Returns the position of the switch at the end of the connector
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
}
