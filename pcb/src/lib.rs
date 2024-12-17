//! The `pcb` crate contains everything required for generating the matrix PCB.

mod footprints;
mod kicad_pcb;
mod matrix_pcb;
mod path;
mod primitives;
mod unit;

pub use matrix_pcb::MatrixPcb;
