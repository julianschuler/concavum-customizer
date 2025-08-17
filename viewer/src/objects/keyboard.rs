use gui::{DisplaySettings, Meshes};
use three_d::{Camera, Context, Light, Mat4, RenderTarget, SquareMatrix};

use crate::objects::{InstancedObject, Render};

/// A keyboard consisting of two halves and bottom plates.
pub struct Keyboard {
    case: InstancedObject,
    bottom_plate: InstancedObject,
    show_case: bool,
    show_bottom_plate: bool,
}

impl Keyboard {
    /// Creates a new keyboard from the given meshes and color.
    pub fn new(context: &Context, display_settings: &DisplaySettings, meshes: &Meshes) -> Self {
        let color = display_settings.colors.keyboard;
        let mirror = Mat4::from_nonuniform_scale(-1.0, 1.0, 1.0);

        let case =
            InstancedObject::new(context, &meshes.case, color, vec![Mat4::identity(), mirror]);
        let bottom_plate = InstancedObject::new(
            context,
            &meshes.bottom_plate,
            color,
            vec![Mat4::identity(), mirror],
        );

        Self {
            case,
            bottom_plate,
            show_case: display_settings.preview.show_keyboard,
            show_bottom_plate: display_settings.preview.show_bottom_plate,
        }
    }
}

impl Render for Keyboard {
    fn render(&self, render_target: &RenderTarget, camera: &Camera, lights: &[&dyn Light]) {
        if self.show_case {
            self.case.render(render_target, camera, lights);
        }

        if self.show_bottom_plate {
            self.bottom_plate.render(render_target, camera, lights);
        }
    }

    fn update_display_settings(&mut self, display_settings: &DisplaySettings) {
        let color = display_settings.colors.keyboard;

        self.case.update_color(color);
        self.bottom_plate.update_color(color);

        self.show_case = display_settings.preview.show_keyboard;
        self.show_bottom_plate = display_settings.preview.show_bottom_plate;
    }
}
