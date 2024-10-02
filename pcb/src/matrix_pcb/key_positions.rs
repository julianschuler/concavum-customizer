use model::matrix_pcb::MatrixPcb as Model;

use crate::primitives::Position;

pub struct KeyPositions {
    columns: Vec<Vec<Position>>,
    thumb_keys: Vec<Position>,
}

impl KeyPositions {
    pub fn from_model(model: &Model, home_row_index: usize) -> Self {
        let columns = Vec::new();
        let thumb_keys = Vec::new();

        Self {
            columns,
            thumb_keys,
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = &Position> {
        self.columns.iter().flatten().chain(self.thumb_keys.iter())
    }
}
