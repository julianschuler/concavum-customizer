use crate::kicad_pcb::{KicadPcb, Net};

/// The nets of the matrix PCB.
pub struct Nets {
    /// The nets for the rows.
    pub rows: Vec<Net>,
    /// The nets for the columns.
    pub columns: Vec<Net>,
}

impl Nets {
    /// Creates and adds the nets to the matrix PCB.
    pub fn create(pcb: &mut KicadPcb) -> Self {
        const MAXIMUM_ROWS: usize = 6;
        const MAXIMUM_COLUMNS: usize = 6;

        let rows: Vec<_> = (1..=MAXIMUM_ROWS)
            .map(|index| pcb.create_net(format!("ROW{index}")))
            .collect();
        let columns: Vec<_> = (1..=MAXIMUM_COLUMNS)
            .map(|index| pcb.create_net(format!("COL{index}")))
            .collect();

        Self { rows, columns }
    }
}
