use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::spawn,
    time::Instant,
};

use config::Config;
use model::{MeshSettings, Model};

use crate::{
    model::{Mesh, Meshes},
    update::{Update, Updater},
};

/// A reloader for reloading a model from a given configuration.
pub struct ModelReloader {
    updater: Updater,
    cancellation_token: CancellationToken,
    cache: Arc<Mutex<HashMap<Config, Meshes>>>,
}

impl ModelReloader {
    /// Creates a new model reloader using the given updater.
    pub fn new(updater: Updater) -> Self {
        Self {
            updater,
            cancellation_token: CancellationToken::new(),
            cache: Arc::default(),
        }
    }

    /// Reloads a model from the given configuration.
    pub fn reload(&mut self, config: Config) {
        self.cancellation_token.cancel();

        let model = Model::from_config(config.clone());

        if let Some(meshes) = self.cache.lock().unwrap().get(&config).cloned() {
            self.updater
                .send_update(Update::New((&model).into(), meshes));
        } else {
            let cancellation_token = CancellationToken::new();
            self.cancellation_token = cancellation_token.clone();

            let start = Instant::now();

            self.updater.send_update(Update::Settings((&model).into()));

            let cancellation_token = self.cancellation_token.clone();
            let updater = self.updater.clone();
            let cache = self.cache.clone();

            spawn(move || {
                let mesh_settings = model.mesh_settings_preview();

                // Preview
                let mut cancelled = false;
                for depth in 0..mesh_settings.depth {
                    let mesh_settings = MeshSettings {
                        depth,
                        bounds: mesh_settings.bounds,
                        threads: mesh_settings.threads,
                    };
                    let mesh = model.mesh_preview(mesh_settings);

                    cancelled = cancellation_token.cancelled();
                    if cancelled {
                        break;
                    } else if mesh.triangle_count() > 0 {
                        updater.send_update(Update::Preview(mesh));
                    }
                }

                // Final Mesh
                if !cancelled {
                    let meshes = model.meshes();

                    cache.lock().unwrap().insert(config, meshes.clone());

                    if !cancellation_token.cancelled() {
                        updater.send_update(Update::Meshes(meshes));

                        eprintln!("Reloaded model in {:?}", start.elapsed());
                    }
                }
            });
        }
    }
}

/// A cancellation token to indicate cancellation between threads.
#[derive(Clone)]
struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates a new cancellation token.
    fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Cancels the cancellation token.
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Returns true if the token was cancelled.
    fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}
