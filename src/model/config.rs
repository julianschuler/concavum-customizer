use serde::Deserialize;
use std::{fs::read_to_string, io, path::Path};

#[derive(Deserialize)]
pub struct Config {
    pub width: f64,
    pub length: f64,
    pub height: f64,
    pub triangulation_tolerance: Option<f64>,
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
