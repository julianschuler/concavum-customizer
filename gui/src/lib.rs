//! The `gui` crate contains everything related to displaying the GUI for changing the configuration.

mod model;
mod reload;
mod update;

use config::Config;
use show::Show;
use three_d::{
    egui::{Align2, Area, SidePanel, Spinner},
    Context, FrameInput, RenderTarget, GUI,
};

use reload::ModelReloader;

pub use model::{Meshes, Settings};
pub use update::{Update, Updater};

/// A graphical user interface for changing the configuration.
pub struct Gui {
    inner: GUI,
    config: Config,
    model_reloader: ModelReloader,
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
        model_reloader.reload(config.clone());

        Self {
            inner,
            config,
            model_reloader,
        }
    }

    /// Updates the GUI using the given frame input.
    pub fn update(&mut self, frame_input: &mut FrameInput, show_spinner: bool) {
        self.inner.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |context| {
                if show_spinner {
                    Area::new("area")
                        .anchor(Align2::RIGHT_BOTTOM, [-15.0, -15.0])
                        .show(context, |ui| ui.add(Spinner::new().size(32.0)));
                }

                SidePanel::left("side_panel")
                    .exact_width(Self::SIDE_PANEL_WIDTH)
                    .show_separator_line(false)
                    .show(context, |ui| {
                        if self.config.show(ui) {
                            self.model_reloader.reload(self.config.clone());
                        }
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
