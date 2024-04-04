#![warn(clippy::pedantic)]

mod config;
mod model;
mod viewer;

use std::path::Path;

use color_eyre::{config::HookBuilder, eyre::WrapErr, Result};

use viewer::{FileWatcher, Window};

fn main() -> Result<()> {
    HookBuilder::new().display_env_section(false).install()?;

    let config_path = Path::new("config.toml");

    let window = Window::try_new()?;

    let reloader = FileWatcher::try_new(config_path, window.model_updater())
        .wrap_err("Failed to initialize file watcher")?;
    let _watcher = reloader.watch().wrap_err("Failed to watch file")?;

    window.run_render_loop();

    Ok(())
}
