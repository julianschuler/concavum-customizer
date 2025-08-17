mod interface_pcbs;
mod keyboard;
mod keyboard_preview;
mod keys;
mod matrix_pcbs;

use config::Color;
use gui::DisplaySettings;
use three_d::{Camera, Context, CpuMesh, Gm, InstancedMesh, Instances, Light, Mat4, RenderTarget};

use crate::material::Physical;

pub use interface_pcbs::InterfacePcbs;
pub use keyboard::Keyboard;
pub use keyboard_preview::KeyboardPreview;
pub use keys::Keys;
pub use matrix_pcbs::MatrixPcbs;

/// A trait for rendering an object.
pub trait Render {
    /// Renders `self` to the given render target.
    fn render(&self, render_target: &RenderTarget, camera: &Camera, lights: &[&dyn Light]);

    /// Updates the display settings of `self`.
    fn update_display_settings(&mut self, display_settings: &DisplaySettings);
}

/// An instanced object which can be rendered in a scene.
struct InstancedObject {
    inner: Gm<InstancedMesh, Physical>,
}

impl InstancedObject {
    /// Creates a new instanced object from a mesh, color and a vector of transformations.
    fn new(context: &Context, mesh: &CpuMesh, color: Color, transformations: Vec<Mat4>) -> Self {
        let instanced_mesh = InstancedMesh::new(
            context,
            &Instances {
                transformations,
                ..Default::default()
            },
            mesh,
        );
        let material = Physical::new(color);

        Self {
            inner: Gm::new(instanced_mesh, material),
        }
    }

    /// Creates a new instanced object from an instanced mesh and color.
    fn from_instanced_mesh(
        context: &Context,
        instanced_mesh: gui::InstancedMesh,
        color: Color,
    ) -> Self {
        let gui::InstancedMesh {
            mesh,
            transformations,
        } = instanced_mesh;

        let instanced_mesh = InstancedMesh::new(
            context,
            &Instances {
                transformations,
                ..Default::default()
            },
            &mesh,
        );
        let material = Physical::new(color);

        Self {
            inner: Gm::new(instanced_mesh, material),
        }
    }

    /// Updates the color of the instanced object.
    fn update_color(&mut self, color: Color) {
        self.inner.material.update(color);
    }
}

impl Render for InstancedObject {
    fn render(&self, render_target: &RenderTarget, camera: &Camera, lights: &[&dyn Light]) {
        render_target.render(camera, &self.inner, lights);
    }

    fn update_display_settings(&mut self, _display_settings: &DisplaySettings) {}
}
