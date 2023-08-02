use std::sync::Arc;

use color_eyre::Report;
use three_d::{
    degrees, vec3, Camera, ClearState, Color, Context, CpuMaterial, DirectionalLight, Event,
    FrameOutput, Gm, InnerSpace, InstancedMesh, Instances, Light, Mesh, MetricSpace, MouseButton,
    OrbitControl, PhysicalMaterial, RenderTarget, WindowError, WindowSettings,
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
        let context = self.window.gl();

        let mut camera = Camera::new_perspective(
            self.window.viewport(),
            vec3(60.00, 50.0, 60.0), // camera position
            vec3(0.0, 0.0, 0.0),     // camera target
            vec3(0.0, 0.0, 1.0),     // camera up
            degrees(45.0),
            0.1,
            1000.0,
        );
        let mut control = OrbitControl::new(vec3(0.0, 0.0, 0.0), 1.0, 1000.0);
        let mut scene = Scene::default();

        let light1 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, -0.5, -0.5));
        let light2 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, 0.5, 0.5));

        self.window.render_loop(move |mut frame_input| {
            camera.set_viewport(frame_input.viewport);
            control.handle_events(&mut camera, &mut frame_input.events);
            scene.handle_events(&mut camera, &mut frame_input.events, &context);

            let screen = frame_input.screen();
            scene.render(&camera, &[&light1, &light2], &screen);

            FrameOutput::default()
        })
    }
}

#[derive(Default)]
struct Scene {
    objects: Vec<Gm<Mesh, PhysicalMaterial>>,
    instanced_objects: Vec<Gm<InstancedMesh, PhysicalMaterial>>,
    background_color: Color,
}

impl Scene {
    fn from_mesh_model(context: &Context, model: &MeshModel) -> Scene {
        let mut objects = Vec::new();
        let mut instanced_objects = Vec::new();

        for object in model.objects.iter() {
            let material = &CpuMaterial {
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

        Scene {
            objects,
            instanced_objects,
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
                        let speed = 0.001 * camera.position().distance(vec3(0.0, 0.0, 0.0));

                        camera.translate(&(speed * translation));
                    }
                }
                Event::UserEvent(model_update) => match model_update {
                    Ok(model) => *self = Scene::from_mesh_model(&context, model),
                    Err(err) => eprintln!("Error:{:?}", Report::from(err.to_owned())),
                },
                _ => (),
            }
        }
    }

    fn render(&self, camera: &Camera, lights: &[&dyn Light], screen: &RenderTarget) {
        let [r, g, b, a] = self.background_color.to_rgba_slice();

        screen.clear(ClearState::color_and_depth(r, g, b, a, 1.0));
        screen.render(&camera, &self.objects, lights);
        screen.render(&camera, &self.instanced_objects, lights);
    }
}
