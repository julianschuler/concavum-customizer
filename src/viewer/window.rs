use std::sync::Arc;

use color_eyre::Report;
use three_d::{
    degrees, vec3, AmbientLight, Camera, ClearState, Color, Context, CpuMaterial, Degrees,
    DirectionalLight, Event, FrameOutput, Gm, InnerSpace, InstancedMesh, Instances, Light, Mesh,
    MouseButton, OrbitControl, PhysicalMaterial, RenderTarget, Vec3, WindowError, WindowSettings,
};
use winit::event_loop::{EventLoopBuilder, EventLoopProxy};

use crate::model::Error;

use super::model::MeshModel;

pub type ModelUpdate = Result<MeshModel, Arc<Error>>;

pub struct Window {
    window: three_d::Window<ModelUpdate>,
    event_loop_proxy: EventLoopProxy<ModelUpdate>,
}

impl Window {
    pub fn try_new() -> Result<Self, WindowError> {
        let event_loop = EventLoopBuilder::with_user_event().build();
        let event_loop_proxy = event_loop.create_proxy();
        let window = three_d::Window::from_event_loop(
            WindowSettings {
                title: "Concavum customizer".to_owned(),
                ..Default::default()
            },
            event_loop,
        )?;

        Ok(Self {
            window,
            event_loop_proxy,
        })
    }

    pub fn event_loop_proxy(&self) -> EventLoopProxy<ModelUpdate> {
        self.event_loop_proxy.clone()
    }

    pub fn run_render_loop(self) {
        const DEFAULT_DISTANCE: f32 = 300.0;
        const DEFAULT_FOV: Degrees = degrees(22.5);
        const DEFAULT_TARGET: Vec3 = vec3(0.0, 0.0, 0.0);

        let context = self.window.gl();

        let mut camera = Camera::new_perspective(
            self.window.viewport(),
            vec3(0.3, -1.0, 1.0).normalize_to(DEFAULT_DISTANCE),
            DEFAULT_TARGET, // camera target
            Vec3::unit_z(), // camera up
            DEFAULT_FOV,
            0.1,
            10000.0,
        );
        let mut control = OrbitControl::new(DEFAULT_TARGET, 1.0, 1000.0);
        let mut scene = Scene::default();

        self.window.render_loop(move |mut frame_input| {
            camera.set_viewport(frame_input.viewport);
            control.handle_events(&mut camera, &mut frame_input.events);
            scene.handle_events(&mut camera, &mut frame_input.events, &context);

            let screen = frame_input.screen();
            scene.render(&camera, &screen);

            FrameOutput::default()
        })
    }
}

#[derive(Default)]
struct Scene {
    objects: Vec<Gm<Mesh, PhysicalMaterial>>,
    instanced_objects: Vec<Gm<InstancedMesh, PhysicalMaterial>>,
    lights: Vec<DirectionalLight>,
    ambient: AmbientLight,
    background_color: Color,
}

impl Scene {
    fn from_mesh_model(context: &Context, model: &MeshModel) -> Scene {
        let mut objects = Vec::new();
        let mut instanced_objects = Vec::new();

        for object in model.objects.iter() {
            let material = CpuMaterial {
                albedo: object.color,
                ..Default::default()
            };
            let material = PhysicalMaterial::new(context, &material);

            if let Some(transformations) = &object.transformations {
                let mesh = InstancedMesh::new(
                    context,
                    &Instances {
                        transformations: transformations.to_owned(),
                        ..Default::default()
                    },
                    &object.mesh,
                );
                instanced_objects.push(Gm::new(mesh, material));
            } else {
                let mesh = Mesh::new(context, &object.mesh);
                objects.push(Gm::new(mesh, material));
            }
        }

        let ambient = AmbientLight::new(context, 0.05, Color::WHITE);
        let lights = model
            .light_directions
            .iter()
            .map(|direction| DirectionalLight::new(context, 1.0, Color::WHITE, direction))
            .collect();

        Scene {
            objects,
            instanced_objects,
            lights,
            ambient,
            background_color: model.background_color,
        }
    }

    fn handle_events(
        &mut self,
        camera: &mut Camera,
        events: &mut [Event<ModelUpdate>],
        context: &Context,
    ) {
        for event in events {
            match event {
                Event::MouseMotion { button, delta, .. } => {
                    if *button == Some(MouseButton::Right) {
                        let right = camera.right_direction().normalize();
                        let up = right.cross(camera.view_direction());
                        let translation = -delta.0 as f32 * right + delta.1 as f32 * up;
                        let speed = 0.001 * camera.position().magnitude();

                        camera.translate(&(speed * translation));
                    }
                }
                Event::UserEvent(model_update) => match model_update {
                    Ok(model) => *self = Scene::from_mesh_model(context, model),
                    Err(err) => eprintln!("Error:{:?}", Report::from(err.to_owned())),
                },
                _ => (),
            }
        }
    }

    fn render(&self, camera: &Camera, screen: &RenderTarget) {
        let [r, g, b, a] = self.background_color.to_rgba_slice();

        let mut lights: Vec<_> = self
            .lights
            .iter()
            .map(|light| light as &dyn Light)
            .collect();
        lights.push(&self.ambient as &dyn Light);

        screen.clear(ClearState::color_and_depth(r, g, b, a, 1.0));
        screen.render(camera, &self.objects, &lights);
        screen.render(camera, &self.instanced_objects, &lights);
    }
}
