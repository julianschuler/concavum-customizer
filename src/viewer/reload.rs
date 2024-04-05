use std::{sync::Arc, time::Instant};

use crate::{
    config::{Config, Error},
    model::Model,
    viewer::{
        window::{ModelUpdater, ReloadEvent},
        Viewable,
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
                let reload_event = match Model::try_from_config(config) {
                    Ok(model) => {
                        let model = model.into_model();
                        eprintln!("Reloaded model in {:?}", start.elapsed());

                        ReloadEvent::Finished(model)
                    }
                    Err(error) => ReloadEvent::Error(Arc::new(error)),
                };

                self.updater.send_event(reload_event);
            }
            Err(error) => self
                .updater
                .send_event(ReloadEvent::Error(Arc::new(error.into()))),
        }
    }
}
