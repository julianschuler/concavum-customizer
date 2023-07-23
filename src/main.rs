mod model;
mod viewer;

use std::path::Path;

use viewer::{
    reload::{self, ModelReloader},
    window::{self, Window},
};

fn main() -> Result<(), Error> {
    let config_path = Path::new("config.toml");

    let window = Window::try_new()?;

    let reloader = ModelReloader::try_new(config_path, window.event_loop_proxy())?;
    let _watcher = reloader.watch()?;

    window.run_event_loop()?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error initializing window
    #[error("Error initializing window")]
    WindowInit(#[from] window::Error),

    /// Error initializing model reloader
    #[error("Error initializing model reloader")]
    ReloadInit(#[from] reload::Error),
}
