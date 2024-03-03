#![allow(unused)]

mod operations;
mod shapes;
mod vector;

type Result<T> = std::result::Result<T, fidget::Error>;

pub use operations::Csg;
pub use shapes::*;
