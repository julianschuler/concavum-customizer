pub mod config;

use std::path::Path;

use opencascade_sys::ffi::{new_point, BRepPrimAPI_MakeBox_ctor, TopoDS_Shape_to_owned};

pub use crate::viewer::model::ViewableModel;
use crate::viewer::model::{Color, Component, Shape};

use config::Config;

pub struct Model {
    shape: Shape,
}

impl Model {
    pub fn try_from_config(config_path: &Path) -> Result<Self, Error> {
        let config = Config::try_from_path(config_path)?;

        let origin = new_point(0.0, 0.0, 0.0);
        let mut cube =
            BRepPrimAPI_MakeBox_ctor(&origin, config.width, config.length, config.height);
        let shape = TopoDS_Shape_to_owned(cube.pin_mut().Shape());

        Ok(Self { shape })
    }
}

impl ViewableModel for Model {
    fn components(self) -> Vec<Component> {
        vec![Component::new(self.shape, Color::default())]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error parsing config
    #[error("Error parsing config")]
    ConfigParsing(#[from] config::Error),
}
