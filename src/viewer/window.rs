use std::sync::Arc;

use color_eyre::Report;
use three_d::{
    degrees, vec3, AmbientLight, Attenuation, Camera, ClearState, Context, CpuMaterial, Degrees,
    FrameInputGenerator, Gm, InnerSpace, InstancedMesh, Instances, Light, Mesh, MouseButton,
    OrbitControl, PhysicalMaterial, PointLight, RenderTarget, Srgba, SurfaceSettings, Vec3,
    Viewport, WindowError, WindowedContext,
};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};

use crate::{model::Error, viewer::model::MeshModel};

pub type ModelUpdate = Result<MeshModel, Arc<Error>>;

pub struct Window {
    event_loop: EventLoop<ModelUpdate>,
    window: winit::window::Window,
    context: WindowedContext,
}

impl Window {
    pub fn try_new() -> Result<Self, WindowError> {
        let event_loop = EventLoopBuilder::with_user_event().build();
        let window = WindowBuilder::new()
            .with_title("Concavum customizer")
            .build(&event_loop)?;
        let context = WindowedContext::from_winit_window(&window, SurfaceSettings::default())?;

        Ok(Self {
            event_loop,
            window,
            context,
        })
    }

    pub fn event_loop_proxy(&self) -> EventLoopProxy<ModelUpdate> {
        self.event_loop.create_proxy()
    }

    pub fn run_render_loop(self) {
        const DEFAULT_DISTANCE: f32 = 300.0;
        const DEFAULT_FOV: Degrees = degrees(22.5);
        const DEFAULT_TARGET: Vec3 = vec3(0.0, 0.0, 0.0);

        let (width, height): (u32, u32) = self.window.inner_size().into();
        let mut camera = Camera::new_perspective(
            Viewport::new_at_origo(width, height),
            vec3(0.3, -1.0, 1.0).normalize_to(DEFAULT_DISTANCE),
            DEFAULT_TARGET, // camera target
            Vec3::unit_z(), // camera up
            DEFAULT_FOV,
            0.1,
            10000.0,
        );
        let mut control = OrbitControl::new(DEFAULT_TARGET, 1.0, 1000.0);
        let mut scene = Scene::default();

        let mut frame_input_generator = FrameInputGenerator::from_winit_window(&self.window);
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let mut frame_input = frame_input_generator.generate(&self.context);

                    control.handle_events(&mut camera, &mut frame_input.events);
                    camera.set_viewport(frame_input.viewport);
                    scene.handle_events(&mut camera, &mut frame_input.events);

                    let screen = frame_input.screen();
                    scene.render(&camera, &screen);

                    self.context.swap_buffers().unwrap();
                    *control_flow = ControlFlow::Poll;
                    self.window.request_redraw();
                }
                Event::WindowEvent { ref event, .. } => {
                    frame_input_generator.handle_winit_window_event(event);
                    match event {
                        winit::event::WindowEvent::Resized(physical_size) => {
                            self.context.resize(*physical_size);
                        }
                        winit::event::WindowEvent::ScaleFactorChanged {
                            new_inner_size, ..
                        } => {
                            self.context.resize(**new_inner_size);
                        }
                        winit::event::WindowEvent::CloseRequested => {
                            control_flow.set_exit();
                        }
                        _ => (),
                    }
                }
                Event::UserEvent(model_update) => match model_update {
                    Ok(model) => scene = Scene::from_mesh_model(&self.context, &model),
                    Err(err) => eprintln!("Error:{:?}", Report::from(err.to_owned())),
                },
                _ => {}
            });
    }
}

#[derive(Default)]
struct Scene {
    objects: Vec<Gm<Mesh, PhysicalMaterial>>,
    instanced_objects: Vec<Gm<InstancedMesh, PhysicalMaterial>>,
    lights: Vec<PointLight>,
    ambient: AmbientLight,
    background_color: Srgba,
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
            background_color: model.background_color,
        }
    }

    fn handle_events(&mut self, camera: &mut Camera, events: &mut [three_d::Event]) {
        for event in events {
            match event {
                three_d::Event::MouseMotion { button, delta, .. } => {
                    if *button == Some(MouseButton::Right) {
                        let right = camera.right_direction().normalize();
                        let up = right.cross(camera.view_direction());
                        let translation = -delta.0 as f32 * right + delta.1 as f32 * up;
                        let speed = 0.001 * camera.position().magnitude();

                        camera.translate(&(speed * translation));
                    }
                }
                _ => (),
            }
        }
    }

    fn render(&self, camera: &Camera, screen: &RenderTarget) {
        let Srgba { r, g, b, a } = self.background_color;

        let mut lights: Vec<_> = self
            .lights
            .iter()
            .map(|light| light as &dyn Light)
            .collect();
        lights.push(&self.ambient as &dyn Light);

        screen
            .clear(ClearState::color_and_depth(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
                1.0,
            ))
            .render(camera, &self.objects, &lights)
            .render(camera, &self.instanced_objects, &lights);
    }
}
