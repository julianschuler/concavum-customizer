use config::KeySize;
use gui::DisplaySettings;
use three_d::{Camera, Context, Light, Mat4, RenderTarget};

use crate::{
    assets::Assets,
    objects::{InstancedObject, Render},
};

/// A set of keys.
pub struct Keys {
    switches: InstancedObject,
    finger_keycaps: InstancedObject,
    thumb_keycaps: InstancedObject,
    show: bool,
}

impl Keys {
    /// Creates a new set of keys.
    pub fn new(
        context: &Context,
        assets: &Assets,
        display_settings: &DisplaySettings,
        switch_positions: Vec<Mat4>,
        finger_key_positions: Vec<Mat4>,
        thumb_key_positions: Vec<Mat4>,
        thumb_key_size: KeySize,
    ) -> Self {
        let colors = &display_settings.colors;

        let switches =
            InstancedObject::new(context, &assets.switch, colors.switch, switch_positions);
        let finger_keycaps = InstancedObject::new(
            context,
            &assets.keycap_1u,
            colors.keycap,
            finger_key_positions,
        );
        let thumb_keycap_asset = match thumb_key_size {
            KeySize::U1_5 => &assets.keycap_1_5u,
            KeySize::U1 => &assets.keycap_1u,
        };
        let thumb_keycaps = InstancedObject::new(
            context,
            thumb_keycap_asset,
            colors.keycap,
            thumb_key_positions,
        );

        Self {
            switches,
            finger_keycaps,
            thumb_keycaps,
            show: display_settings.preview.show_keys,
        }
    }
}

impl Render for Keys {
    fn render(&self, render_target: &RenderTarget, camera: &Camera, lights: &[&dyn Light]) {
        if self.show {
            self.switches.render(render_target, camera, lights);
            self.finger_keycaps.render(render_target, camera, lights);
            self.thumb_keycaps.render(render_target, camera, lights);
        }
    }

    fn update_display_settings(&mut self, display_settings: &DisplaySettings) {
        let colors = &display_settings.colors;

        self.switches.update_color(colors.switch);
        self.finger_keycaps.update_color(colors.keycap);
        self.thumb_keycaps.update_color(colors.keycap);

        self.show = display_settings.preview.show_keys;
    }
}
