use config::Config;
use model::{
    matrix_pcb::{MatrixPcb as Model, THICKNESS},
    KeyPositions,
};

use crate::{kicad_pcb::KicadPcb, unit::IntoUnit};

/// A builder for the matrix PCB.
pub struct Builder {
    pcb: KicadPcb,
    cluster_connector_index: usize,
    home_row_index: usize,
}

impl Builder {
    pub fn from_config(config: &Config) -> Self {
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

    pub fn build(self) -> KicadPcb {
        self.pcb
    }
}
