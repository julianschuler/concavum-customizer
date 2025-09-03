use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

#[cfg(not(target_arch = "wasm32"))]
use std::thread::spawn;

use fidget::render::CancelToken;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use config::Config;
use model::Model;
use web_time::Instant;

use crate::{
    model::{make_settings, Mesh, Meshes},
    update::{Update, Updater},
};

macro_rules! info {
    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!($($arg)*);
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

/// A reloader for reloading a model from a given configuration.
pub struct ModelReloader {
    updater: Updater,
    sender: Sender<ReloadTask>,
    previous_config: Option<Config>,
    cancel_token: CancelToken,
    cache: Cache,
}

/// A reload task to be performed in a separate thread.
struct ReloadTask {
    model: Model,
    config: Config,
    cancel_token: CancelToken,
}

/// A cache for already meshed configurations.
type Cache = Arc<Mutex<HashMap<Config, Meshes>>>;

impl ModelReloader {
    /// Creates a new model reloader using the given updater.
    pub fn new(updater: Updater) -> Self {
        let (sender, receiver) = channel();
        let cache: Cache = Arc::default();

        Self::spawn_reload_thread(receiver, updater.clone(), cache.clone());

        Self {
            updater,
            sender,
            previous_config: None,
            cancel_token: CancelToken::new(),
            cache,
        }
    }

    /// Reloads a model from the given configuration.
    pub fn reload(&mut self, config: &Config) {
        if self
            .previous_config
            .as_ref()
            .is_some_and(|previous_config| previous_config == config)
        {
            self.updater
                .send_update(Update::DisplaySettings(config.into()));
            return;
        }

        self.cancel_token.cancel();
        self.previous_config = Some(config.clone());

        let model = Model::from_config(config);

        if let Some(meshes) = self.cached_meshes(config) {
            self.updater
                .send_update(Update::New(make_settings(&model, config), meshes));
        } else {
            let cancel_token = CancelToken::new();
            self.cancel_token = cancel_token.clone();

            self.updater
                .send_update(Update::Settings(make_settings(&model, config)));

            let _ = self.sender.send(ReloadTask {
                model,
                config: config.clone(),
                cancel_token,
            });
        }
    }

    /// Returns the cached meshes corresponding to the given configuration.
    pub fn cached_meshes(&self, config: &Config) -> Option<Meshes> {
        self.cache
            .lock()
            .expect("the lock should not be poisened")
            .get(config)
            .cloned()
    }

    /// Spawns a thread for handling the actual reloading.
    fn spawn_reload_thread(receiver: Receiver<ReloadTask>, updater: Updater, cache: Cache) {
        spawn(move || 'outer: loop {
            let Ok(ReloadTask {
                model,
                config,
                cancel_token,
            }) = receiver.recv()
            else {
                break;
            };

            let start = Instant::now();

            let mut mesh_settings = model.mesh_settings_preview(cancel_token.clone());

            // Preview
            for depth in 1..mesh_settings.depth {
                mesh_settings.depth = depth;
                let Some(mesh) = model.mesh_preview(&mesh_settings) else {
                    continue 'outer;
                };

                if mesh.triangle_count() > 0 {
                    updater.send_update(Update::Preview(mesh));
                }
            }

            // Final Meshes
            let Some(meshes) = model.meshes(cancel_token) else {
                continue;
            };

            cache
                .lock()
                .expect("the lock should not be poisened")
                .insert(config, meshes.clone());

            updater.send_update(Update::Meshes(meshes));

            info!("Reloaded model in {:?}", start.elapsed());
        });
    }
}

#[cfg(target_arch = "wasm32")]
fn spawn(f: impl FnOnce() + Send + 'static) {
    let options = web_sys::WorkerOptions::new();
    options.set_type(web_sys::WorkerType::Module);
    let worker = web_sys::Worker::new_with_options("./worker.js", &options)
        .expect("should be able to create worker using `worker.js`");

    // Double-boxing because `dyn FnOnce` is unsized and so `Box<dyn FnOnce()>` is a fat pointer.
    // But `Box<Box<dyn FnOnce()>>` is just a plain pointer, and since wasm has 32-bit pointers,
    // we can cast it to a `u32` and back.
    let function_pointer = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce()>));

    let message = web_sys::js_sys::Array::new();
    message.push(&wasm_bindgen::memory());
    message.push(&JsValue::from(function_pointer as u32));

    worker
        .post_message(&message)
        .expect("should be able to send message to worker");
}

/// This function is the entry point called in `worker.js`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn worker_entry_point(addr: u32) {
    // Interpret the address we were given as a pointer to a closure to call.
    let closure = unsafe { Box::from_raw(addr as *mut Box<dyn FnOnce()>) };
    (*closure)();
}
