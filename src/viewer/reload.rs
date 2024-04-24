use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::spawn,
    time::Instant,
};

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
    cancellation_token: CancellationToken,
}

impl ModelReloader {
    /// Creates a new model reloader using the given updater for updating the model.
    pub fn new(updater: ModelUpdater) -> Self {
        Self {
            updater,
            cancellation_token: CancellationToken::new(),
        }
    }

    pub fn reload(&mut self, config: Result<Config, Error>) {
        match config {
            Ok(config) => {
                self.cancellation_token.cancel();
                let cancellation_token = CancellationToken::new();
                self.cancellation_token = cancellation_token.clone();

                let start = Instant::now();
                self.updater.send_event(ReloadEvent::Started);
                let model = Model::from_config(config);

                let mesh_settings = model.mesh_settings();
                let cancellation_token = self.cancellation_token.clone();
                let updater = self.updater.clone();
                spawn(move || {
                    let mut cancelled = false;
                    for depth in 0..mesh_settings.depth {
                        let mesh_settings = Settings {
                            depth,
                            bounds: mesh_settings.bounds,
                            threads: mesh_settings.threads,
                        };
                        let model = model.into_model(mesh_settings);

                        cancelled = cancellation_token.cancelled();
                        if cancelled {
                            break;
                        }
                        updater.send_event(ReloadEvent::Updated(model));
                    }

                    if !cancelled {
                        let model = model.into_model(mesh_settings);

                        if !cancellation_token.cancelled() {
                            updater.send_event(ReloadEvent::Finished(model));

                            eprintln!("Reloaded model in {:?}", start.elapsed());
                        }
                    }
                });
            }
            Err(error) => self.updater.send_event(ReloadEvent::Error(Arc::new(error))),
        }
    }
}

#[derive(Clone)]
struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}
