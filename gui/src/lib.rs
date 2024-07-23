mod config;
mod model;
mod reload;
mod update;

use config::{Config, Show};
use three_d::{
    egui::{Align2, Area, SidePanel, Spinner},
    Context, FrameInput, RenderTarget, GUI,
};

use reload::ModelReloader;

pub use model::{Meshes, Model};
pub use update::{SceneUpdate, SceneUpdater};

/// A graphical user interface for changing the configuration.
pub struct Gui {
    inner: GUI,
    config: Config,
    model_reloader: ModelReloader,
}

impl Gui {
    /// Creates a new GUI for the given context.
    pub fn new(context: &Context, updater: SceneUpdater) -> Self {
        let inner = GUI::new(context);
        let config = Config::default();

        let mut model_reloader = ModelReloader::new(updater);
        model_reloader.reload(Ok(config.clone()));

        Self {
            inner,
            config,
            model_reloader,
        }
    }

    /// Updates the GUI using the given frame input.
    pub fn update(&mut self, frame_input: &mut FrameInput, show_spinner: bool) {
        const SIDE_PANEL_WIDTH: f32 = 250.0;

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
                    .exact_width(SIDE_PANEL_WIDTH)
                    .show_separator_line(false)
                    .show(context, |ui| self.config.show(ui));
            },
        );
    }

    /// Renders the GUI to the given render target.
    pub fn render(&self, render_target: &RenderTarget) {
        render_target
            .write(|| self.inner.render())
            .expect("rendering the gui should never fail");
    }
}
