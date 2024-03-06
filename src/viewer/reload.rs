use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use notify::{
    event::{AccessKind::Close, AccessMode::Write},
    recommended_watcher, Event,
    EventKind::Access,
    RecommendedWatcher, RecursiveMode, Watcher,
};
use winit::event_loop::EventLoopProxy;

use crate::{
    model::{self, Viewable},
    viewer::window::ModelUpdate,
};

/// A file watcher reloading the model upon file change.
pub struct ModelReloader {
    config_path: PathBuf,
    event_loop_proxy: EventLoopProxy<ModelUpdate>,
}

impl ModelReloader {
    /// Creates a new reloader for the given config file path.
    ///
    /// Upon file change, a model update is sent via the given event loop proxy.
    /// Returns [`Error`] if the file path could not be canonicalized.
    pub fn try_new(
        config_path: &Path,
        event_loop_proxy: EventLoopProxy<ModelUpdate>,
    ) -> Result<Self, Error> {
        let config_path = config_path.canonicalize()?;

        Ok(Self {
            config_path,
            event_loop_proxy,
        })
    }

    /// Starts watching the config file in a different thread.
    ///
    /// Returns [`Error`] if watching the file was unsuccessful.
    pub fn watch(self) -> Result<RecommendedWatcher, Error> {
        self.update_model();

        let config_path_parent = self.config_path.parent().unwrap().to_path_buf();

        let mut watcher = recommended_watcher(move |result: notify::Result<Event>| {
            if let Ok(event) = result {
                if matches!(event.kind, Access(Close(Write)))
                    && event.paths.iter().any(|path| path == &self.config_path)
                {
                    self.update_model();
                }
            }
        })?;

        watcher.watch(&config_path_parent, RecursiveMode::NonRecursive)?;

        Ok(watcher)
    }

    /// Updates the model by reloading it from the config file and sending it via the event loop
    /// proxy.
    fn update_model(&self) {
        let start = Instant::now();
        let model_update = match model::Model::try_from_config(&self.config_path) {
            Ok(model) => {
                let model = model.into_model();
                eprintln!("Reloaded model in {:?}", start.elapsed());

                Ok(model)
            }
            Err(error) => Err(Arc::new(error)),
        };
        let _ = self.event_loop_proxy.send_event(model_update);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to canonicalize file path
    #[error("failed to canonicalize file path")]
    Canonilize(#[from] io::Error),
    /// Failed to initialize file watcher
    #[error("failed to initialize file watcher")]
    Notify(#[from] notify::Error),
}
