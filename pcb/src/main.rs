//! This file executes the PCB generation with the default config.

use std::{
    fs::File,
    io::{Result, Write},
};

use config::Config;
use pcb::MatrixPcb;

/// The main function of the PCB generator.
///
/// # Errors
///
/// Returns an I/O error if the PCB file could not be written.
pub fn main() -> Result<()> {
    let config = Config::default();
    let matrix_pcb = MatrixPcb::from_config(&config);
    let pcb_file = matrix_pcb.to_kicad_board();

    let mut file = File::create("/tmp/matrix_pcb.kicad_pcb")?;
    file.write_all(pcb_file.as_bytes())
}
