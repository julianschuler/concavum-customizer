use std::{fs::read_to_string, io, path::Path};

use glam::DVec3;
use hex_color::HexColor;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub points1: Vec<DVec3>,
    pub points2: Vec<DVec3>,
    pub triangulation_tolerance: Option<f64>,
    pub colors: Colors,
}

#[derive(Deserialize)]
pub struct Colors {
    pub keycap: HexColor,
    pub switch: HexColor,
    pub matrix_pcb: HexColor,
    pub interface_pcb: HexColor,
    pub fpc_connector: HexColor,
}

impl Config {
    pub fn try_from_path(config_path: &Path) -> Result<Self, Error> {
        Ok(toml::from_str(&read_to_string(config_path)?)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to open file
    #[error("Failed to open file")]
    FileOpen(#[from] io::Error),

    /// Failed to parse TOML
    #[error("Failed to parse TOML")]
    TomlParse(#[from] toml::de::Error),
}
