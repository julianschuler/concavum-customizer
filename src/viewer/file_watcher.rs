use std::{
    io,
    path::{Path, PathBuf},
};

use notify::{
    event::{AccessKind::Close, AccessMode::Write},
    recommended_watcher, Event,
    EventKind::Access,
    RecommendedWatcher, RecursiveMode, Watcher,
};

use crate::{config::Config, viewer::reload::ModelReloader};

/// A file watcher reloading the model upon file change.
pub struct FileWatcher {
    config_path: PathBuf,
    model_reloader: ModelReloader,
}

impl FileWatcher {
    /// Creates a new reloader for the given config file path.
    ///
    /// Upon file change, the model is reloaded via via the given model reloader.
    /// Returns [`Error`] if the file path could not be canonicalized.
    pub fn try_new(config_path: &Path, model_reloader: ModelReloader) -> Result<Self, Error> {
        let config_path = config_path.canonicalize()?;

        Ok(Self {
            config_path,
            model_reloader,
        })
    }

    /// Starts watching the config file in a different thread.
    ///
    /// Returns [`Error`] if watching the file was unsuccessful.
    pub fn watch(mut self) -> Result<RecommendedWatcher, Error> {
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

    /// Parse the config file and reload the model from it.
    fn update_model(&mut self) {
        let config = Config::try_from_path(&self.config_path);
        self.model_reloader.reload(config);
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
