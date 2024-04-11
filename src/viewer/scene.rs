use hex_color::HexColor;
use three_d::{
    AmbientLight, Attenuation, Camera, ClearState, Context, CpuMesh, Gm, InstancedMesh, Instances,
    Light, Mat4, Mesh, PointLight, RenderTarget, Srgba,
};

use crate::viewer::{material::Physical, model::Model};

/// Fixed assets loaded from OBJ files.
pub struct Assets {
    switch: CpuMesh,
    keycap_1u: CpuMesh,
    keycap_1_5u: CpuMesh,
}

impl Assets {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Default)]
pub struct Scene {
    objects: Vec<Object>,
    instanced_objects: Vec<InstancedObject>,
    lights: Vec<PointLight>,
    ambient: AmbientLight,
    background_color: HexColor,
}

impl Scene {
    /// Creates a scene from a model for the given assets and context.
    pub fn from_model(context: &Context, model: Model, assets: &Assets) -> Scene {
        let switch_positions = model
            .finger_key_positions
            .iter()
            .chain(&model.finger_key_positions)
            .cloned()
            .collect();

        let objects = vec![Object::new(
            &context,
            &model.keyboard,
            model.colors.keyboard,
        )];
        let instanced_objects = vec![
            InstancedObject::new(
                context,
                &assets.switch,
                model.colors.switch,
                switch_positions,
            ),
            InstancedObject::new(
                context,
                &assets.keycap_1u,
                model.colors.keycap,
                model.finger_key_positions,
            ),
            InstancedObject::new(
                context,
                &assets.keycap_1_5u,
                model.colors.keycap,
                model.thumb_key_positions,
            ),
        ];

        let ambient = AmbientLight::new(context, 0.05, Srgba::WHITE);
        let lights = model
            .light_positions
            .iter()
            .map(|direction| {
                PointLight::new(
                    context,
                    1.0,
                    Srgba::WHITE,
                    direction,
                    Attenuation::default(),
                )
            })
            .collect();

        Scene {
            objects,
            instanced_objects,
            lights,
            ambient,
            background_color: model.colors.background,
        }
    }

    /// Renders the scene with a given camera and render target.
    pub fn render(&self, camera: &Camera, screen: &RenderTarget) {
        let HexColor { r, g, b, a } = self.background_color;

        let mut lights: Vec<_> = self
            .lights
            .iter()
            .map(|light| light as &dyn Light)
            .collect();
        lights.push(&self.ambient as &dyn Light);

        screen
            .clear(ClearState::color_and_depth(
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
                f32::from(a) / 255.0,
                1.0,
            ))
            .render(
                camera,
                self.objects.iter().map(|object| &object.inner),
                &lights,
            )
            .render(
                camera,
                self.instanced_objects.iter().map(|object| &object.inner),
                &lights,
            );
    }
}

struct Object {
    inner: Gm<Mesh, Physical>,
}

impl Object {
    fn new(context: &Context, mesh: &CpuMesh, color: HexColor) -> Self {
        let mesh = Mesh::new(context, &mesh);
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
