use crate::kicad_pcb::{KicadPcb, Net};

/// The nets of the matrix PCB.
pub struct Nets {
    /// The nets for the rows.
    rows: Vec<Net>,
    /// The nets for the columns.
    columns: Vec<Net>,
    row_count: usize,
    column_count: usize,
    thumb_switch_count: usize,
    home_row_index: usize,
}

impl Nets {
    /// Creates and adds the nets to the matrix PCB.
    pub fn create(
        pcb: &mut KicadPcb,
        row_count: usize,
        column_count: usize,
        thumb_switch_count: usize,
        home_row_index: usize,
    ) -> Self {
        const MAXIMUM_ROWS: usize = 6;
        const MAXIMUM_COLUMNS: usize = 6;

        let rows: Vec<_> = (1..=MAXIMUM_ROWS)
            .map(|index| pcb.create_net(format!("ROW{index}")))
            .collect();
        let columns: Vec<_> = (1..=MAXIMUM_COLUMNS)
            .map(|index| pcb.create_net(format!("COL{index}")))
            .collect();

        Self {
            rows,
            columns,
            row_count,
            column_count,
            thumb_switch_count,
            home_row_index,
        }
    }

    /// Returns the net of the home row.
    pub fn home_row(&self) -> &Net {
        &self.rows[self.home_row_index + 1]
    }

    /// Returns the column nets of the thumb switches.
    pub fn thumb_row(&self) -> &Net {
        &self.rows[0]
    }

    /// Return the column nets of the thumb switches.
    pub fn thumb_columns(&self) -> &[Net] {
        &self.columns[..self.thumb_switch_count]
    }

    /// Returs the nets of the finger rows.
    pub fn finger_rows(&self) -> &[Net] {
        &self.rows[1..=self.row_count]
    }

    /// Returns the nets of the finger columns.
    pub fn columns(&self) -> &[Net] {
        &self.columns[..self.column_count]
    }

    /// Returns the FFC connector nets.
    pub fn ffc_connector_nets(&self) -> [Net; 12] {
        [
            self.columns[0].clone(),
            self.rows[5].clone(),
            self.rows[4].clone(),
            self.rows[3].clone(),
            self.rows[2].clone(),
            self.rows[1].clone(),
            self.columns[1].clone(),
            self.columns[2].clone(),
            self.columns[3].clone(),
            self.columns[4].clone(),
            self.columns[5].clone(),
            self.rows[0].clone(),
        ]
    }
}
