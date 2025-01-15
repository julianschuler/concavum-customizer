use std::{
    future::Future,
    io::{Cursor, Write},
    sync::mpsc::{channel, Receiver, Sender},
};

use config::Config;
use pcb::MatrixPcb;
use rfd::AsyncFileDialog;
use show::egui::{popup_below_widget, Align, Button, Id, Layout, RichText, Ui, Window};
use three_d::{CpuMesh, Indices, Positions};
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{reload::ModelReloader, Error, Meshes};

type Update = Result<Option<Config>, Error>;

/// A menu for loading/saving configuration and exporting model files.
pub struct FileMenu {
    sender: Sender<Update>,
    receiver: Receiver<Update>,
    error: String,
    show_popup: bool,
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
            show_popup: false,
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

                let export_button = ui.button("Export");

                let popup_id = Id::new("popup");

                // let export_button = ui.add_enabled(!is_reloading, Button::new("Export"));

                if export_button.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }

                popup_below_widget(ui, popup_id, &export_button, |ui| ui.label("Test"));

                if export_button
                    .on_hover_text("Exports all the model files in a ZIP archive")
                    .clicked()
                {
                    self.export_model(ui, config, model_reloader);
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

    /// Exports the
    fn export_model(&mut self, ui: &mut Ui, config: &Config, model_reloader: &ModelReloader) {
        if f64::from(config.keyboard.resolution) > 0.2 {
            self.show_popup = true;
        } else {
            match model_reloader.cached_meshes(config) {
                Some(meshes) => self.spawn_local(export_model(config.clone(), meshes)),
                None => self.error = "The model has not been fully meshed yet".to_string(),
            }
        }
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

/// Loads the config from a TOML file.
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
    let matrix_pcb = MatrixPcb::from_config(&config);
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

    zip.start_file("config.toml", SimpleFileOptions::default())?;
    zip.write_all(toml.as_bytes())?;

    zip.start_file("left_half.stl", SimpleFileOptions::default())?;
    zip.write_stl(meshes.left_half)?;

    zip.start_file("right_half.stl", SimpleFileOptions::default())?;
    zip.write_stl(meshes.right_half)?;

    zip.start_file("bottom_plate.stl", SimpleFileOptions::default())?;
    zip.write_stl(meshes.bottom_plate)?;

    zip.start_file("matrix_pcb.kicad_pcb", SimpleFileOptions::default())?;
    zip.write_all(matrix_pcb.to_kicad_board().as_bytes())?;

    zip.start_file("kikit_parameters.json", SimpleFileOptions::default())?;
    zip.write_all(include_bytes!("kikit_parameters.json"))?;

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

/// A trait for writing a mesh to a binary STL.
trait WriteStl: Write {
    /// Writes the given mesh to a binary STL.
    fn write_stl(&mut self, mesh: CpuMesh) -> Result<(), Error>;
}

impl<W: Write> WriteStl for W {
    fn write_stl(&mut self, mesh: CpuMesh) -> Result<(), Error> {
        const HEADER: &[u8] = b"This is a binary STL file exported by the concavum customizer";

        if let CpuMesh {
            positions: Positions::F32(vertices),
            indices: Indices::U32(indices),
            ..
        } = mesh
        {
            let triangle_count = indices.len() / 3;

            self.write_all(HEADER)?;
            self.write_all(&[0; 80 - HEADER.len()])?;
            #[allow(clippy::cast_possible_truncation)]
            self.write_all(&(triangle_count as u32).to_le_bytes())?;

            for triangle in indices.chunks_exact(3) {
                let a = vertices[triangle[0] as usize];
                let b = vertices[triangle[1] as usize];
                let c = vertices[triangle[2] as usize];

                let normal = (b - a).cross(c - a);

                for vector in [normal, a, b, c] {
                    self.write_all(&vector.x.to_le_bytes())?;
                    self.write_all(&vector.y.to_le_bytes())?;
                    self.write_all(&vector.z.to_le_bytes())?;
                }
                self.write_all(&[0; size_of::<u16>()])?;
            }

            Ok(())
        } else {
            Err(Error::InvalidMesh("The mesh representation is invalid"))
        }
    }
}
