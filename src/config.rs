use serde::Deserialize;
use std::{fs::read_to_string, io, path::Path};

#[derive(Deserialize)]
pub struct Config {
    pub outer: f32,
    pub inner: f32,
    pub height: f32,
}

impl Config {
    pub fn try_from_path(config_path: &Path) -> Result<Self, Error> {
        Ok(toml::from_str(&read_to_string(config_path)?)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error opening file
    #[error("Error opening file")]
    Open(#[from] io::Error),

    // Error parsing file
    #[error("Error parsing file")]
    Parsing(#[from] toml::de::Error),
}
