use std::{
    io,
    path::{Path, PathBuf},
};

use fj_interop::model::Model;
use notify::{
    event::{AccessKind::Close, AccessMode::Write},
    recommended_watcher, Event,
    EventKind::Access,
    RecommendedWatcher, RecursiveMode, Watcher,
};
use winit::event_loop::EventLoopProxy;

use crate::model::{self, ViewableModel};

pub struct ModelReloader {
    config_path: PathBuf,
    event_loop_proxy: EventLoopProxy<Result<Model, model::Error>>,
}

impl ModelReloader {
    pub fn try_new(
        config_path: &Path,
        event_loop_proxy: EventLoopProxy<Result<Model, model::Error>>,
    ) -> Result<Self, Error> {
        let config_path = config_path.canonicalize()?;

        Ok(Self {
            config_path,
            event_loop_proxy,
        })
    }

    pub fn watch(self) -> Result<RecommendedWatcher, Error> {
        self.update_model();

        let config_path_parent = self.config_path.parent().unwrap().to_path_buf();

        let mut watcher = recommended_watcher(move |result: notify::Result<Event>| {
            if let Ok(event) = result {
                if matches!(event.kind, Access(Close(Write))) {
                    if event.paths.iter().any(|path| path == &self.config_path) {
                        self.update_model();
                    }
                }
            }
        })?;

        watcher.watch(&config_path_parent, RecursiveMode::NonRecursive)?;

        Ok(watcher)
    }

    fn update_model(&self) {
        let model =
            model::Model::try_from_config(&self.config_path).map(|model| model.into_mesh_model());
        let _ = self.event_loop_proxy.send_event(model);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to canonicalize file path
    #[error("Failed to canonicalize file path")]
    Canonilize(#[from] io::Error),

    /// Failed to initialize file watcher
    #[error("Failed to initialize file watcher")]
    Notify(#[from] notify::Error),
}
