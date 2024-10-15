use gui::DisplaySettings;
use three_d::{Camera, Context, CpuMesh, Light, Mat4, RenderTarget, SquareMatrix};

use crate::objects::{InstancedObject, Render};

/// A low resolution preview for the actual keyboard.
pub struct KeyboardPreview {
    inner: InstancedObject,
    show: bool,
}

impl KeyboardPreview {
    /// Creates a new pair of interface PCBs.
    pub fn new(context: &Context, mesh: &CpuMesh, display_settings: &DisplaySettings) -> Self {
        let inner = InstancedObject::new(
            context,
            mesh,
            display_settings.colors.keyboard,
            vec![
                Mat4::identity(),
                Mat4::from_nonuniform_scale(-1.0, 1.0, 1.0),
            ],
        );

        Self {
            inner,
            show: display_settings.preview.show_keyboard,
        }
    }
}

impl Render for KeyboardPreview {
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
