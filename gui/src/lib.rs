//! The `gui` crate contains everything related to displaying the GUI for changing the configuration.

mod file_menu;
mod model;
mod reload;
mod update;

use std::{
    fmt::{Display, Formatter, Result as FormattingResult},
    io,
    string::FromUtf8Error,
};

use config::Config;
use file_menu::FileMenu;
use show::{
    egui::{Color32, RichText},
    Show,
};
use three_d::{
    egui::{Align2, Area, SidePanel, Spinner},
    Context, FrameInput, RenderTarget, GUI,
};
use zip::result::ZipError;

use reload::ModelReloader;

pub use model::{DisplaySettings, Meshes, Settings};
pub use update::{Update, Updater};

/// A graphical user interface for changing the configuration.
pub struct Gui {
    inner: GUI,
    config: Config,
    model_reloader: ModelReloader,
    file_menu: FileMenu,
}

impl Gui {
    /// The width of the side panel.
    pub const SIDE_PANEL_WIDTH: f32 = 250.0;

    /// Creates a new GUI using the given updater.
    #[must_use]
    pub fn new(context: &Context, updater: Updater) -> Self {
        let inner = GUI::new(context);
        let config = Config::default();

        let mut model_reloader = ModelReloader::new(updater);
        model_reloader.reload(&config);

        let file_menu = FileMenu::new();

        Self {
            inner,
            config,
            model_reloader,
            file_menu,
        }
    }

    /// Updates the GUI using the given frame input.
    pub fn update(&mut self, frame_input: &mut FrameInput, show_spinner: bool) {
        const MARGIN: f32 = 15.0;

        self.inner.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |context| {
                if show_spinner {
                    Area::new("spinner")
                        .anchor(Align2::RIGHT_BOTTOM, [-MARGIN, -MARGIN])
                        .show(context, |ui| ui.add(Spinner::new().size(32.0)));
                }

                SidePanel::left("side_panel")
                    .exact_width(Self::SIDE_PANEL_WIDTH)
                    .show_separator_line(false)
                    .show(context, |ui| {
                        let mut changed = false;

                        changed |= self
                            .file_menu
                            .show(ui, &mut self.config, &self.model_reloader);
                        changed |= self.config.show(ui);

                        if changed {
                            self.model_reloader.reload(&self.config);
                        }
                    });

                Area::new("error")
                    .anchor(Align2::LEFT_TOP, [MARGIN, MARGIN])
                    .show(context, |ui| {
                        ui.label(
                            RichText::new(self.file_menu.error())
                                .monospace()
                                .color(Color32::LIGHT_RED),
                        )
                    });
            },
        );
    }

    /// Renders the GUI to the given render target.
    ///
    /// # Panics
    ///
    /// Panics if the GUI could not be rendered.
    pub fn render(&self, render_target: &RenderTarget) {
        render_target
            .write(|| self.inner.render())
            .expect("rendering the gui should never fail");
    }
}

/// An error type for errors which can occur when interacting with the GUI.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No file was selected.
    NoFileSelected,
    /// A file contains invalid UTF-8.
    InvalidUtf8(#[from] FromUtf8Error),
    /// An I/O error occurred.
    IoError(#[from] io::Error),
    /// Failed to serialize to TOML.
    TomlSerialize(#[from] toml::ser::Error),
    /// Failed to deserialize from TOML.
    TomlDeserialize(#[from] toml::de::Error),
    /// A mesh is invalid.
    InvalidMesh(&'static str),
    /// Failed to create the ZIP archive.
    ZipError(#[from] ZipError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormattingResult {
        match self {
            Error::NoFileSelected => Ok(()),
            Error::IoError(inner) => inner.fmt(f),
            Error::InvalidUtf8(inner) => inner.fmt(f),
            Error::TomlSerialize(inner) => inner.fmt(f),
            Error::TomlDeserialize(inner) => inner.fmt(f),
            Error::InvalidMesh(inner) => inner.fmt(f),
            Error::ZipError(inner) => inner.fmt(f),
        }
    }
}
