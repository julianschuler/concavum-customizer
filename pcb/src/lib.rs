//! The `pcb` crate contains everything required for generating the matrix PCB.

mod kicad_pcb;
mod matrix_pcb;
mod primitives;
mod serializer;
mod unit;

pub use matrix_pcb::MatrixPcb;
