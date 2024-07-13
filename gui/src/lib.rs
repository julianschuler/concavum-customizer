use three_d::{
    egui::{Align2, Area, Spinner},
    Context, FrameInput, RenderTarget, GUI,
};

/// A graphical user interface for changing the configuration.
pub struct Gui {
    inner: GUI,
}

impl Gui {
    /// Creates a new GUI for the given context.
    pub fn new(context: &Context) -> Self {
        let inner = GUI::new(context);

        Self { inner }
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
