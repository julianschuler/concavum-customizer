//! Test application

use std::{
    fs::File,
    io::{Result, Write},
};

use config::Config;
use pcb::MatrixPcb;

/// Entry point
fn main() -> Result<()> {
    let config = Config::default();
    let pcb = MatrixPcb::from_config(&config);
    let kicad_board = pcb.to_kicad_board();

    let mut file = File::create("/tmp/matrix_pcb.kicad_pcb")?;
    file.write_all(kicad_board.as_bytes())?;

    Ok(())
}
