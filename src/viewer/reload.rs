use std::{sync::Arc, time::Instant};

use fidget::mesh::Settings;

use crate::{
    config::{Config, Error},
    model::Model,
    viewer::{
        model::IntoShape,
        window::{ModelUpdater, ReloadEvent},
    },
};

pub struct ModelReloader {
    updater: ModelUpdater,
}

impl ModelReloader {
    /// Creates a new model reloader using the given updater for updating the model.
    pub fn new(updater: ModelUpdater) -> Self {
        Self { updater }
    }

    pub fn reload(&self, config: Result<Config, Error>) {
        match config {
            Ok(config) => {
                let start = Instant::now();

                self.updater.send_event(ReloadEvent::Started);
                let model = Model::from_config(config);

                let mesh_settings = model.mesh_settings();
                for depth in 0..mesh_settings.depth {
                    let mesh_settings = Settings {
                        depth,
                        bounds: mesh_settings.bounds,
                        threads: mesh_settings.threads,
                    };
                    let model = model.into_model(mesh_settings);
                    self.updater.send_event(ReloadEvent::Updated(model));
                }

                let model = model.into_model(mesh_settings);
                self.updater.send_event(ReloadEvent::Finished(model));

                eprintln!("Reloaded model in {:?}", start.elapsed());
            }
            Err(error) => self.updater.send_event(ReloadEvent::Error(Arc::new(error))),
        }
    }
}
