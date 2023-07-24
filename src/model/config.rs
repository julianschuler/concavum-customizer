use std::{fs::read_to_string, io, path::Path};

use glam::{DVec2, DVec3};
use hex_color::HexColor;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub points1: Vec<DVec3>,
    pub points2: Vec<DVec3>,

    pub preview: Preview,
    pub finger_cluster: FingerCluster,
    pub thumb_cluster: ThumbCluster,
    pub keyboard: Keyboard,
    pub colors: Colors,
}

#[derive(Deserialize)]
pub struct Preview {
    pub show_keys: bool,
    pub show_interface_pcb: bool,
    pub show_bottom_plate: bool,
    pub triangulation_tolerance: Option<f64>,
}

#[derive(Deserialize)]
pub struct FingerCluster {
    pub rows: u8,
    pub columns: Vec<Column>,
    pub side_angles: (f64, f64),
}

#[derive(Deserialize)]
pub struct Column {
    pub curvature_angle: f64,
    pub offset: DVec2,
}

#[derive(Deserialize)]
pub struct ThumbCluster {
    pub keys: u8,
    pub curvature: f64,
    pub rotation: DVec3,
    pub offset: DVec3,
}

#[derive(Deserialize)]
pub struct Keyboard {
    pub tilting_angle: DVec2,
    pub shell_thickness: f64,
    pub bottom_plate_thickness: f64,
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
