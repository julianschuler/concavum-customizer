use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::spawn,
    time::Instant,
};

use fidget::mesh::Settings;
use three_d::CpuMesh;

use crate::{
    config::{Config, Error},
    model::Model,
    viewer::{
        model::Mesh,
        window::{SceneUpdate, SceneUpdater},
    },
};

pub struct ModelReloader {
    updater: SceneUpdater,
    cancellation_token: CancellationToken,
    cache: Arc<Mutex<HashMap<Config, CpuMesh>>>,
}

impl ModelReloader {
    /// Creates a new model reloader using the given updater for updating the model.
    pub fn new(updater: SceneUpdater) -> Self {
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

                self.updater
                    .send_update(SceneUpdate::Model((&model).into()));

                if let Some(mesh) = self.cache.lock().unwrap().get(&config).cloned() {
                    self.updater.send_update(SceneUpdate::Mesh(mesh));
                } else {
                    let cancellation_token = CancellationToken::new();
                    self.cancellation_token = cancellation_token.clone();

                    let start = Instant::now();

                    let cancellation_token = self.cancellation_token.clone();
                    let updater = self.updater.clone();
                    let cache = self.cache.clone();

                    spawn(move || {
                        let mesh_settings = model.mesh_settings_preview();

                        // Preview
                        let mut cancelled = false;
                        for depth in 0..mesh_settings.depth {
                            let mesh_settings = Settings {
                                depth,
                                bounds: mesh_settings.bounds,
                                threads: mesh_settings.threads,
                            };
                            let mesh = model.mesh_preview(mesh_settings);

                            cancelled = cancellation_token.cancelled();
                            if cancelled {
                                break;
                            }
                            updater.send_update(SceneUpdate::Preview(mesh));
                        }

                        // Final Mesh
                        if !cancelled {
                            let mesh = model.mesh();

                            cache.lock().unwrap().insert(config, mesh.clone());

                            if !cancellation_token.cancelled() {
                                updater.send_update(SceneUpdate::Mesh(mesh));

                                eprintln!("Reloaded model in {:?}", start.elapsed());
                            }
                        }
                    });
                }
            }
            Err(error) => self
                .updater
                .send_update(SceneUpdate::Error(Arc::new(error))),
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
