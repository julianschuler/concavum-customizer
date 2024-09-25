use config::Config;
use model::{
    matrix_pcb::{MatrixPcb as Model, THICKNESS},
    KeyPositions,
};

use crate::{kicad_pcb::KicadPcb, unit::IntoUnit};

/// A PCB connecting the keys to each other in a matrix.
pub struct MatrixPcb(KicadPcb);

impl MatrixPcb {
    /// Creates a new matrix PCB from the given configuration.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        MatrixPcbBuilder::from_config(config).build()
    }

    /// Serializes the matrix PCB to the KiCAD board file format.
    #[must_use]
    pub fn to_kicad_board(&self) -> String {
        self.0.to_board_file()
    }
}

/// A builder for the matrix PCB.
struct MatrixPcbBuilder {
    pcb: KicadPcb,
    cluster_connector_index: usize,
    home_row_index: usize,
}

impl MatrixPcbBuilder {
    fn from_config(config: &Config) -> Self {
        let key_positions = KeyPositions::from_config(config);
        let _model = Model::from_positions(&key_positions);

        let pcb = KicadPcb::new(THICKNESS.mm());

        let cluster_connector_index =
            usize::from(config.finger_cluster.columns.left_side_column.active);
        #[allow(clippy::cast_sign_loss)]
        let home_row_index = key_positions.columns.home_row_index as usize;

        Self {
            pcb,
            cluster_connector_index,
            home_row_index,
        }
    }

    fn build(self) -> MatrixPcb {
        MatrixPcb(self.pcb)
    }
}
