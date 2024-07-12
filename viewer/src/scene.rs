use config::Colors;
use glam::DVec3;
use hex_color::HexColor;
use three_d::{
    AmbientLight, Attenuation, Camera, ClearState, Context, CpuMesh, Gm, InstancedMesh, Instances,
    Light, Mat4, Mesh, PointLight, RenderTarget, SquareMatrix, Srgba,
};

use crate::{
    assets::Assets,
    material::Physical,
    model::{Meshes, Model},
};

#[derive(Default)]
pub struct Scene {
    keyboard: Option<Object>,
    preview: Option<InstancedObject>,
    instanced_objects: Vec<InstancedObject>,
    lights: Vec<PointLight>,
    ambient: AmbientLight,
    colors: Colors,
    show_bottom_plate: bool,
}

impl Scene {
    /// Creates a scene from a model for the given assets and context.
    pub fn from_model(context: &Context, model: Model, assets: &Assets) -> Scene {
        let switch_positions = model
            .finger_key_positions
            .iter()
            .chain(&model.thumb_key_positions)
            .copied()
            .collect();
        let colors = model.colors.clone();

        let mut instanced_objects = if model.settings.show_keys {
            vec![
                InstancedObject::new(context, &assets.switch, colors.switch, switch_positions),
                InstancedObject::new(
                    context,
                    &assets.keycap_1u,
                    colors.keycap,
                    model.finger_key_positions,
                ),
                InstancedObject::new(
                    context,
                    &assets.keycap_1_5u,
                    colors.keycap,
                    model.thumb_key_positions,
                ),
            ]
        } else {
            Vec::new()
        };
        if model.settings.show_interface_pcb {
            instanced_objects.push(InstancedObject::new(
                context,
                &assets.interface_pcb,
                colors.interface_pcb,
                model.interface_pcb_positions,
            ));
        }

        let ambient = AmbientLight::new(context, 0.05, Srgba::WHITE);
        let lights = model
            .settings
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
        let show_bottom_plate = model.settings.show_bottom_plate;

        Scene {
            keyboard: None,
            preview: None,
            instanced_objects,
            lights,
            ambient,
            colors,
            show_bottom_plate,
        }
    }

    /// Updates the preview.
    pub fn update_preview(&mut self, context: &Context, preview: &CpuMesh) {
        self.preview = Some(InstancedObject::new(
            context,
            preview,
            self.colors.keyboard,
            vec![
                Mat4::identity(),
                Mat4::from_nonuniform_scale(-1.0, 1.0, 1.0),
            ],
        ));
    }

    /// Updates the objects using the given meshes.
    pub fn update_objects(&mut self, context: &Context, meshes: &Meshes) {
        if self.keyboard.is_none() {
            self.keyboard = Some(Object::new(context, &meshes.keyboard, self.colors.keyboard));
        }

        if self.show_bottom_plate {
            let transforms = vec![
                Mat4::identity(),
                Mat4::from_nonuniform_scale(-1.0, 1.0, 1.0),
            ];

            self.instanced_objects.push(InstancedObject::new(
                context,
                &meshes.bottom_plate,
                self.colors.keyboard,
                transforms,
            ));
        }
    }

    /// Renders the scene with a given camera and render target.
    pub fn render(&self, camera: &Camera, screen: &RenderTarget) {
        let HexColor { r, g, b, a } = self.colors.background;

        let mut lights: Vec<_> = self
            .lights
            .iter()
            .map(|light| light as &dyn Light)
            .collect();
        lights.push(&self.ambient as &dyn Light);

        let render_target = screen
            .clear(ClearState::color_and_depth(
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
                f32::from(a) / 255.0,
                1.0,
            ))
            .render(
                camera,
                self.instanced_objects.iter().map(|object| &object.inner),
                &lights,
            );

        if let Some(keyboard) = &self.keyboard {
            render_target.render(camera, &keyboard.inner, &lights);
        } else if let Some(preview) = &self.preview {
            render_target.render(camera, &preview.inner, &lights);
        }
    }
}

struct Object {
    inner: Gm<Mesh, Physical>,
}

impl Object {
    fn new(context: &Context, mesh: &CpuMesh, color: HexColor) -> Self {
        let mesh = Mesh::new(context, mesh);
        let material = Physical::new(color);

        Self {
            inner: Gm::new(mesh, material),
        }
    }
}

struct InstancedObject {
    inner: Gm<InstancedMesh, Physical>,
}

impl InstancedObject {
    fn new(context: &Context, mesh: &CpuMesh, color: HexColor, transformations: Vec<Mat4>) -> Self {
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
}
