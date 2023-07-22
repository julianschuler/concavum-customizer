mod config;
mod model;
mod window;

use std::path::Path;

use fj_interop::model::Model;
use notify::{
    event::{AccessKind::Close, AccessMode::Write},
    recommended_watcher, Event,
    EventKind::Access,
    RecommendedWatcher, RecursiveMode, Watcher,
};
use winit::event_loop::EventLoopProxy;

use window::Window;

fn reload_model_on_config_change(
    config_path: &Path,
    event_loop_proxy: EventLoopProxy<Result<Model, model::Error>>,
) -> notify::Result<RecommendedWatcher> {
    let config_path_parent = config_path.parent().unwrap().to_path_buf();
    let config_path = config_path.canonicalize().unwrap();

    let mut watcher = recommended_watcher(move |result: notify::Result<Event>| {
        if let Ok(event) = result {
            if matches!(event.kind, Access(Close(Write))) {
                if event.paths.iter().any(|path| path == &config_path) {
                    let model =
                        model::Model::try_from_config(&config_path).map(|model| model.into());
                    let _ = event_loop_proxy.send_event(model);
                }
            }
        }
    })?;

    watcher.watch(&config_path_parent, RecursiveMode::NonRecursive)?;

    Ok(watcher)
}

fn main() -> Result<(), Error> {
    let config_path = Path::new("config.toml");

    let window = Window::new()?;
    let event_loop_proxy = window.event_loop_proxy();

    let model = model::Model::try_from_config(&config_path).map(|model| model.into());
    let _ = event_loop_proxy.send_event(model);

    let mut watcher = reload_model_on_config_change(config_path, event_loop_proxy)?;

    window.run_event_loop()?;

    watcher.unwatch(config_path)?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error initializing window
    #[error("Error initializing main loop")]
    WindowInit(#[from] window::Error),

    /// Error initializing graphics
    #[error("Error initializing graphics")]
    Notify(#[from] notify::Error),
}
