pub mod config;

use std::path::Path;

use glam::DVec3;
use opencascade::primitives::{Face, Surface};
use opencascade_sys::ffi;

pub use crate::viewer::model::ViewableModel;
use crate::viewer::model::{Color, Component};
use config::Config;

pub struct Model {
    components: Vec<Component>,
    triangulation_tolerance: f64,
}

impl Model {
    pub fn try_from_config(config_path: &Path) -> Result<Self, Error> {
        let config = Config::try_from_path(config_path)?;
        let triangulation_tolerance = config.triangulation_tolerance.unwrap_or(0.001);

        let mut components = Vec::new();

        let points1: Vec<_> = config
            .points1
            .into_iter()
            .map(|point| DVec3::from_array(point))
            .collect();

        let points2: Vec<_> = config
            .points2
            .into_iter()
            .map(|point| DVec3::from_array(point))
            .collect();

        let surface = Surface::bezier(&[&points1, &points2]);
        let face = Face::from_surface(&surface);

        let shape = ffi::cast_face_to_shape(&face.inner);
        let shape = ffi::TopoDS_Shape_to_owned(shape);

        components.push(Component::new(shape, Color([0, 255, 0, 255])));

        Ok(Self {
            components,
            triangulation_tolerance,
        })
    }
}

impl ViewableModel for Model {
    fn triangulation_tolerance(&self) -> f64 {
        self.triangulation_tolerance
    }

    fn components(self) -> Vec<Component> {
        self.components
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to parse config
    #[error("Failed to parse config")]
    ParseConfig(#[from] config::Error),
}
