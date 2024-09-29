use std::iter::once;

use gui::DisplaySettings;
use three_d::{Camera, Context, Light, Mat4, RenderTarget};

use crate::{
    assets::Assets,
    objects::{InstancedObject, Render},
};

/// A pair of matrix PCBs, one for each side.
pub struct MatrixPcbs {
    objects: Vec<InstancedObject>,
    show: bool,
}

impl MatrixPcbs {
    /// Creates a new pair of matrix PCBs.
    pub fn new(
        context: &Context,
        assets: &Assets,
        display_settings: &DisplaySettings,
        meshes: Vec<gui::InstancedMesh>,
        switch_positions: Vec<Mat4>,
        fpc_pad_positions: Vec<Mat4>,
    ) -> Self {
        let color = display_settings.colors.matrix_pcb;

        let objects = meshes
            .into_iter()
            .map(|mesh| InstancedObject::from_instanced_mesh(context, mesh, color))
            .chain(once(InstancedObject::new(
                context,
                &assets.matrix_pcb_pad,
                color,
                switch_positions,
            )))
            .chain(once(InstancedObject::new(
                context,
                &assets.fpc_pad,
                color,
                fpc_pad_positions,
            )))
            .collect();

        Self {
            objects,
            show: display_settings.preview.show_matrix_pcb,
        }
    }
}

impl Render for MatrixPcbs {
    fn render(&self, render_target: &RenderTarget, camera: &Camera, lights: &[&dyn Light]) {
        if self.show {
            for object in &self.objects {
                object.render(render_target, camera, lights);
            }
        }
    }

    fn update_display_settings(&mut self, display_settings: &DisplaySettings) {
        for object in &mut self.objects {
            object.update_color(display_settings.colors.matrix_pcb);
        }

        self.show = display_settings.preview.show_matrix_pcb;
    }
}
