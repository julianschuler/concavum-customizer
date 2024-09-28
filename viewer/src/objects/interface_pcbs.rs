use gui::DisplaySettings;
use three_d::{Camera, Context, Light, Mat4, RenderTarget};

use crate::{
    assets::Assets,
    objects::{InstancedObject, Render},
};

/// A pair of interface PCBs, one for each side.
pub struct InterfacePcbs {
    inner: InstancedObject,
    show: bool,
}

impl InterfacePcbs {
    /// Creates a new pair of interface PCBs.
    pub fn new(
        context: &Context,
        assets: &Assets,
        display_settings: &DisplaySettings,
        positions: Vec<Mat4>,
    ) -> Self {
        let inner = InstancedObject::new(
            context,
            &assets.interface_pcb,
            display_settings.colors.interface_pcb,
            positions,
        );

        Self {
            inner,
            show: display_settings.preview.show_interface_pcb,
        }
    }
}

impl Render for InterfacePcbs {
    fn render(&self, render_target: &RenderTarget, camera: &Camera, lights: &[&dyn Light]) {
        if self.show {
            self.inner.render(render_target, camera, lights);
        }
    }

    fn update_display_settings(&mut self, display_settings: &DisplaySettings) {
        self.inner
            .update_color(display_settings.colors.interface_pcb);

        self.show = display_settings.preview.show_interface_pcb;
    }
}
