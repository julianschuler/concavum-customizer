use std::{
    future::Future,
    io::{Cursor, Write},
    sync::mpsc::{channel, Receiver, Sender},
};

use config::Config;
use rfd::AsyncFileDialog;
use show::egui::{Align, Button, Layout, RichText, Ui};
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{model::WriteStl, reload::ModelReloader, Error, Meshes};

type Update = Result<Option<Config>, Error>;

/// A menu for loading/saving configuration and exporting model files.
pub struct FileMenu {
    sender: Sender<Update>,
    receiver: Receiver<Update>,
    error: String,
}

impl FileMenu {
    /// Creates a new file menu.
    #[must_use]
    pub fn new() -> Self {
        let (sender, receiver) = channel();

        Self {
            sender,
            receiver,
            error: String::new(),
        }
    }

    /// Shows a widget for the file menu. Returns true if the configuration was changed.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        config: &mut Config,
        model_reloader: &ModelReloader,
        is_reloading: bool,
    ) -> bool {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Configuration").strong().size(16.0));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Reverse order since widgets are placed right to left
                if ui
                    .add_enabled(!is_reloading, Button::new("Export"))
                    .on_hover_text("Exports all the model files in a ZIP archive")
                    .clicked()
                {
                    match model_reloader.cached_meshes(config) {
                        Some(meshes) => self.spawn_local(export_model(config.clone(), meshes)),
                        None => self.error = "The model has not been fully meshed yet".to_string(),
                    }
                }
                if ui
                    .button("Save")
                    .on_hover_text("Saves the configuration as a TOML file")
                    .clicked()
                {
                    self.spawn_local(save_config(config.clone()));
                }
                if ui
                    .button("Load")
                    .on_hover_text("Loads the configuration from a TOML file")
                    .clicked()
                {
                    self.spawn_local(load_config());
                }
            });
        });

        if let Ok(update) = self.receiver.try_recv() {
            match update {
                Ok(update) => {
                    self.error = String::new();

                    if let Some(config_update) = update {
                        *config = config_update;
                        return true;
                    }
                }
                Err(error) => self.error = error.to_string(),
            }
        }

        false
    }

    /// Returns the current error to display.
    pub fn error(&self) -> &str {
        &self.error
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn_local(&self, future: impl Future<Output = Update> + 'static) {
        let sender = self.sender.clone();

        let future = async move {
            let _ = sender.send(future.await);
        };

        wasm_bindgen_futures::spawn_local(future);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_local(&self, future: impl Future<Output = Update> + 'static + Send) {
        let sender = self.sender.clone();

        let future = async move {
            let _ = sender.send(future.await);
        };

        std::thread::spawn(|| pollster::block_on(future));
    }
}

/// Loads the config from a TOML file
async fn load_config() -> Update {
    let file = AsyncFileDialog::new()
        .add_filter("toml", &["toml"])
        .pick_file()
        .await
        .ok_or(Error::NoFileSelected)?;

    let bytes = file.read().await;

    let string = String::from_utf8(bytes)?;

    Ok(Some(toml::from_str(&string)?))
}

/// Saves the config to a TOML file.
async fn save_config(config: Config) -> Update {
    let toml = toml::to_string(&config)?;

    let file = AsyncFileDialog::new()
        .add_filter("toml", &["toml"])
        .set_file_name("config.toml")
        .save_file()
        .await
        .ok_or(Error::NoFileSelected)?;

    file.write(toml.as_bytes()).await?;

    Ok(None)
}

/// Exports all the model files in a ZIP archive.
async fn export_model(config: Config, meshes: Meshes) -> Update {
    let toml = toml::to_string(&config)?;
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

    zip.start_file("config.toml", SimpleFileOptions::default())?;
    zip.write_all(toml.as_bytes())?;

    zip.start_file("keyboard.stl", SimpleFileOptions::default())?;
    zip.write_stl(meshes.keyboard)?;

    zip.start_file("bottom_plate.stl", SimpleFileOptions::default())?;
    zip.write_stl(meshes.bottom_plate)?;

    let data = zip.finish()?;

    let file = AsyncFileDialog::new()
        .add_filter("zip", &["zip"])
        .set_file_name("concavum.zip")
        .save_file()
        .await
        .ok_or(Error::NoFileSelected)?;

    file.write(&data.into_inner()).await?;

    Ok(None)
}
