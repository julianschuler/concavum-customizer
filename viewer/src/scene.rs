use std::iter::once;

use config::{Color, Colors};
use glam::DVec3;
use gui::{DisplaySettings, Meshes, Settings};
use three_d::{
    AmbientLight, Attenuation, Camera, ClearState, Context, CpuMesh, Gm, InstancedMesh, Instances,
    Light, Mat4, Mesh, PointLight, RenderTarget, SquareMatrix, Srgba,
};

use crate::{assets::Assets, material::Physical};

/// A scene rendering a keyboard model.
pub struct Scene {
    keyboard: Option<Keyboard>,
    preview: Option<InstancedObject>,
    keys: Keys,
    interface_pcb: InstancedObject,
    matrix_pcb: MatrixPcbs,
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
            .chain(&settings.thumb_key_positions)
            .copied()
            .collect();
        let display_settings = settings.display_settings.clone();

        let keys = Keys::new(
            context,
            assets,
            &display_settings,
            switch_positions.clone(),
            settings.finger_key_positions,
            settings.thumb_key_positions,
        );
        let interface_pcb = InstancedObject::new(
            context,
            &assets.interface_pcb,
            display_settings.colors.interface_pcb,
            settings.interface_pcb_positions,
        );
        let matrix_pcb = MatrixPcbs::new(
            context,
            settings.matrix_pcb_meshes,
            assets,
            display_settings.colors.matrix_pcb,
            switch_positions,
            settings.fpc_pad_positions,
        );

        let ambient = AmbientLight::new(context, 0.05, Srgba::WHITE);
        let lights = display_settings
            .preview
            .light_positions
            .iter()
            .map(|direction| {
                PointLight::new(
                    context,
                    1.0,
                    Srgba::WHITE,
                    &DVec3::from(*direction).as_vec3().to_array().into(),
                    Attenuation::default(),
                )
            })
            .collect();

        Scene {
            keyboard: None,
            preview: None,
            keys,
            interface_pcb,
            matrix_pcb,
            lights,
            ambient,
            display_settings,
        }
    }

    /// Updates the preview using the given mesh.
    pub fn update_preview(&mut self, context: &Context, preview: &CpuMesh) {
        self.preview = Some(InstancedObject::new(
            context,
            preview,
            self.display_settings.colors.keyboard,
            vec![
                Mat4::identity(),
                Mat4::from_nonuniform_scale(-1.0, 1.0, 1.0),
            ],
        ));
    }

    /// Updates the objects using the given meshes.
    pub fn update_keyboard(&mut self, context: &Context, meshes: &Meshes) {
        self.keyboard = Some(Keyboard::new(
            context,
            meshes,
            self.display_settings.colors.keyboard,
        ));
    }

    /// Updates the scene using the given display settings.
    pub fn update_display_settings(&mut self, display_settings: DisplaySettings) {
        if let Some(keyboard) = &mut self.keyboard {
            keyboard.update_color(display_settings.colors.keyboard);
        }
        if let Some(preview) = &mut self.preview {
            preview.update_color(display_settings.colors.keyboard);
        }
        self.keys.update_colors(&display_settings.colors);
        self.matrix_pcb
            .update_color(display_settings.colors.matrix_pcb);
        self.interface_pcb
            .update_color(display_settings.colors.interface_pcb);

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
            keyboard.render(
                render_target,
                camera,
                &lights,
                self.display_settings.preview.show_bottom_plate,
            );
        } else if let Some(preview) = &self.preview {
            render_target.render(camera, &preview.inner, &lights);
        }

        if self.display_settings.preview.show_keys {
            self.keys.render(render_target, camera, &lights);
        }

        if self.display_settings.preview.show_interface_pcb {
            render_target.render(camera, &self.interface_pcb.inner, &lights);
        }

        if self.display_settings.preview.show_matrix_pcb {
            self.matrix_pcb.render(render_target, camera, &lights);
        }
    }
}

/// An object which can be rendered in a scene.
struct Object {
    inner: Gm<Mesh, Physical>,
}

impl Object {
    /// Creates a new object from a mesh and color.
    fn new(context: &Context, mesh: &CpuMesh, color: Color) -> Self {
        let mesh = Mesh::new(context, mesh);
        let material = Physical::new(color);

        Self {
            inner: Gm::new(mesh, material),
        }
    }

    /// Updates the color of the object.
    fn update_color(&mut self, color: Color) {
        self.inner.material.update(color);
    }
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

struct Keyboard {
    left_half: Object,
    right_half: Object,
    bottom_plate: InstancedObject,
}
impl Keyboard {
    /// Creates a new keyboard from the given meshes and color.
    fn new(context: &Context, meshes: &Meshes, color: Color) -> Self {
        let left_half = Object::new(context, &meshes.left_half, color);
        let right_half = Object::new(context, &meshes.right_half, color);

        let bottom_plate = InstancedObject::new(
            context,
            &meshes.bottom_plate,
            color,
            vec![
                Mat4::identity(),
                Mat4::from_nonuniform_scale(-1.0, 1.0, 1.0),
            ],
        );

        Self {
            left_half,
            right_half,
            bottom_plate,
        }
    }

    /// Updates the color of the keyboard.
    fn update_color(&mut self, color: Color) {
        self.left_half.update_color(color);
        self.right_half.update_color(color);
        self.bottom_plate.update_color(color);
    }

    /// Renders the keyboard to the given render target.
    fn render(
        &self,
        render_target: &RenderTarget,
        camera: &Camera,
        lights: &[&dyn Light],
        show_bottom_plate: bool,
    ) {
        render_target.render(camera, &self.left_half.inner, lights);
        render_target.render(camera, &self.right_half.inner, lights);

        if show_bottom_plate {
            render_target.render(camera, &self.bottom_plate.inner, lights);
        }
    }
}

/// A set of keys to display.
struct Keys {
    switches: InstancedObject,
    finger_keycaps: InstancedObject,
    thumb_keycaps: InstancedObject,
}

impl Keys {
    /// Creates a new set of keys.
    fn new(
        context: &Context,
        assets: &Assets,
        display_settings: &DisplaySettings,
        switch_positions: Vec<Mat4>,
        finger_key_positions: Vec<Mat4>,
        thumb_key_positions: Vec<Mat4>,
    ) -> Self {
        let switches = InstancedObject::new(
            context,
            &assets.switch,
            display_settings.colors.switch,
            switch_positions,
        );
        let finger_keycaps = InstancedObject::new(
            context,
            &assets.keycap_1u,
            display_settings.colors.keycap,
            finger_key_positions,
        );
        let thumb_keycaps = InstancedObject::new(
            context,
            &assets.keycap_1_5u,
            display_settings.colors.keycap,
            thumb_key_positions,
        );

        Self {
            switches,
            finger_keycaps,
            thumb_keycaps,
        }
    }

    /// Updates the colors of the keys.
    fn update_colors(&mut self, colors: &Colors) {
        self.switches.update_color(colors.switch);
        self.finger_keycaps.update_color(colors.keycap);
        self.thumb_keycaps.update_color(colors.keycap);
    }

    /// Renders the keys to the given render target.
    fn render(&self, render_target: &RenderTarget, camera: &Camera, lights: &[&dyn Light]) {
        render_target.render(
            camera,
            [
                &self.switches.inner,
                &self.finger_keycaps.inner,
                &self.thumb_keycaps.inner,
            ],
            lights,
        );
    }
}

/// Two matrix PCBs, one for each side.
struct MatrixPcbs {
    objects: Vec<InstancedObject>,
}

impl MatrixPcbs {
    /// Creates a new set of matrix PCBs.
    fn new(
        context: &Context,
        meshes: Vec<gui::InstancedMesh>,
        assets: &Assets,
        color: Color,
        switch_postitions: Vec<Mat4>,
        fpc_pad_positions: Vec<Mat4>,
    ) -> Self {
        let objects = meshes
            .into_iter()
            .map(|mesh| InstancedObject::from_instanced_mesh(context, mesh, color))
            .chain(once(InstancedObject::new(
                context,
                &assets.matrix_pcb_pad,
                color,
                switch_postitions,
            )))
            .chain(once(InstancedObject::new(
                context,
                &assets.fpc_pad,
                color,
                fpc_pad_positions,
            )))
            .collect();

        Self { objects }
    }

    /// Updates the color of the matrix PCBs.
    fn update_color(&mut self, color: Color) {
        for object in &mut self.objects {
            object.update_color(color);
        }
    }

    /// Renders the matrix PCBs to the given render target.
    fn render(&self, render_target: &RenderTarget, camera: &Camera, lights: &[&dyn Light]) {
        render_target.render(
            camera,
            self.objects.iter().map(|object| &object.inner),
            lights,
        );
    }
}
