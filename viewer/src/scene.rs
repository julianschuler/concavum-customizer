use config::Color;
use gui::{DisplaySettings, Meshes, Settings};
use three_d::{
    AmbientLight, Attenuation, Camera, ClearState, Context, CpuMesh, Light, PointLight,
    RenderTarget, Srgba,
};

use crate::{
    assets::Assets,
    objects::{InterfacePcbs, Keyboard, KeyboardPreview, Keys, MatrixPcbs, Render},
};

/// A scene rendering a keyboard model.
pub struct Scene {
    keyboard: Option<Keyboard>,
    keyboard_preview: Option<KeyboardPreview>,
    keys: Keys,
    interface_pcbs: InterfacePcbs,
    matrix_pcbs: MatrixPcbs,
    lights: Vec<PointLight>,
    ambient: AmbientLight,
    display_settings: DisplaySettings,
}

impl Scene {
    /// Creates a scene from the given model settings using the given assets.
    pub fn from_settings(context: &Context, settings: Settings, assets: &Assets) -> Scene {
        let switch_positions: Vec<_> = settings
            .finger_key_positions
            .iter()
            .chain(&settings.thumb_key_settings.thumb_key_positions)
            .copied()
            .collect();
        let display_settings = &settings.display_settings;

        let keys = Keys::new(
            context,
            assets,
            display_settings,
            switch_positions.clone(),
            settings.finger_key_positions,
            settings.thumb_key_settings.thumb_key_positions,
            settings.thumb_key_settings.key_size,
        );
        let interface_pcbs = InterfacePcbs::new(
            context,
            assets,
            display_settings,
            settings.interface_pcb_positions,
        );
        let matrix_pcbs = MatrixPcbs::new(
            context,
            assets,
            display_settings,
            settings.matrix_pcb_meshes,
            switch_positions,
            settings.fpc_pad_positions,
        );

        let ambient = AmbientLight::new(context, 0.05, Srgba::WHITE);
        let lights = settings
            .light_positions
            .iter()
            .map(|position| {
                PointLight::new(context, 0.8, Srgba::WHITE, position, Attenuation::default())
            })
            .collect();

        Scene {
            keyboard: None,
            keyboard_preview: None,
            keys,
            interface_pcbs,
            matrix_pcbs,
            lights,
            ambient,
            display_settings: settings.display_settings,
        }
    }

    /// Updates the preview using the given mesh.
    pub fn update_preview(&mut self, context: &Context, preview: &CpuMesh) {
        self.keyboard_preview = Some(KeyboardPreview::new(
            context,
            preview,
            &self.display_settings,
        ));
    }

    /// Updates the keyboard using the given meshes.
    pub fn update_keyboard(&mut self, context: &Context, meshes: &Meshes) {
        self.keyboard = Some(Keyboard::new(context, &self.display_settings, meshes));
    }

    /// Updates the scene using the given display settings.
    pub fn update_display_settings(&mut self, display_settings: DisplaySettings) {
        if let Some(keyboard) = &mut self.keyboard {
            keyboard.update_display_settings(&display_settings);
        }
        if let Some(preview) = &mut self.keyboard_preview {
            preview.update_display_settings(&display_settings);
        }
        self.keys.update_display_settings(&display_settings);
        self.interface_pcbs
            .update_display_settings(&display_settings);
        self.matrix_pcbs.update_display_settings(&display_settings);

        self.display_settings = display_settings;
    }

    /// Renders the scene with a given camera and render target.
    pub fn render(&self, camera: &Camera, render_target: &RenderTarget) {
        let Color { r, g, b, a } = self.display_settings.colors.background;

        let mut lights: Vec<_> = self
            .lights
            .iter()
            .map(|light| light as &dyn Light)
            .collect();
        lights.push(&self.ambient as &dyn Light);

        let render_target = render_target.clear(ClearState::color_and_depth(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
            1.0,
        ));

        if let Some(keyboard) = &self.keyboard {
            keyboard.render(render_target, camera, &lights);
        } else if let Some(preview) = &self.keyboard_preview {
            preview.render(render_target, camera, &lights);
        }

        self.keys.render(render_target, camera, &lights);
        self.interface_pcbs.render(render_target, camera, &lights);
        self.matrix_pcbs.render(render_target, camera, &lights);
    }
}
