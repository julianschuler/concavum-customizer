use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::spawn,
    time::Instant,
};

use config::{Config, Error};
use model::{MeshSettings, Model};

use crate::{
    model::{Mesh, Meshes},
    update::{Update, Updater},
};

pub struct ModelReloader {
    updater: Updater,
    cancellation_token: CancellationToken,
    cache: Arc<Mutex<HashMap<Config, Meshes>>>,
}

impl ModelReloader {
    /// Creates a new model reloader using the given updater for updating the model.
    pub fn new(updater: Updater) -> Self {
        Self {
            updater,
            cancellation_token: CancellationToken::new(),
            cache: Arc::default(),
        }
    }

    pub fn reload(&mut self, config: Result<Config, Error>) {
        match config {
            Ok(config) => {
                self.cancellation_token.cancel();

                let model = Model::from_config(config.clone());

                if let Some(meshes) = self.cache.lock().unwrap().get(&config).cloned() {
                    self.updater
                        .send_update(Update::New((&model).into(), meshes));
                } else {
                    let cancellation_token = CancellationToken::new();
                    self.cancellation_token = cancellation_token.clone();

                    let start = Instant::now();

                    self.updater.send_update(Update::Model((&model).into()));

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
            Err(error) => self.updater.send_update(Update::Error(Arc::new(error))),
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
