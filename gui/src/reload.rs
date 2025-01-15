use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

#[cfg(not(target_arch = "wasm32"))]
use std::thread::spawn;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use config::Config;
use model::{MeshSettings, Model};
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
    previous_config: Option<Config>,
    cancellation_token: CancellationToken,
    cache: Arc<Mutex<HashMap<Config, Meshes>>>,
}

impl ModelReloader {
    /// Creates a new model reloader using the given updater.
    pub fn new(updater: Updater) -> Self {
        Self {
            updater,
            previous_config: None,
            cancellation_token: CancellationToken::new(),
            cache: Arc::default(),
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

        self.cancellation_token.cancel();
        self.previous_config = Some(config.clone());

        let model = Model::from_config(config);

        if let Some(meshes) = self.cached_meshes(config) {
            self.updater
                .send_update(Update::New(make_settings(&model, config), meshes));
        } else {
            let cancellation_token = CancellationToken::new();
            self.cancellation_token = cancellation_token.clone();

            let start = Instant::now();

            self.updater
                .send_update(Update::Settings(make_settings(&model, config)));
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

#[cfg(target_arch = "wasm32")]
fn spawn(f: impl FnOnce() + Send + 'static) {
    let worker = web_sys::Worker::new("./worker.js")
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

/// This fucntion is the entry point called in `worker.js`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn worker_entry_point(addr: u32) {
    // Interpret the address we were given as a pointer to a closure to call.
    let closure = unsafe { Box::from_raw(addr as *mut Box<dyn FnOnce()>) };
    (*closure)();
}
