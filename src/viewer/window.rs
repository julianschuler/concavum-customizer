use std::sync::Arc;

use color_eyre::Report;
use three_d::{
    degrees, vec3, AmbientLight, Attenuation, Camera, ClearState, Context, CpuMaterial, Degrees,
    FrameInputGenerator, Gm, InnerSpace, InstancedMesh, Instances, Light, Mesh, MouseButton,
    OrbitControl, PhysicalMaterial, PointLight, RenderTarget, Srgba, SurfaceSettings, Vec3,
    Viewport, WindowError, WindowedContext,
};
use winit::{
    event::{Event, WindowEvent},
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
        let frame_input_generator = FrameInputGenerator::from_winit_window(&self.window);
        let (width, height): (u32, u32) = self.window.inner_size().into();
        let viewport = Viewport::new_at_origo(width, height);
        let mut application = Application::new(frame_input_generator, viewport);

        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    application.handle_redraw(&self.context);

                    self.context.swap_buffers().unwrap();
                    control_flow.set_poll();
                    self.window.request_redraw();
                }
                Event::WindowEvent { ref event, .. } => {
                    application.handle_window_event(event, &self.context, control_flow);
                }
                Event::UserEvent(model_update) => {
                    application.handle_model_update(model_update, &self.context);
                }
                _ => {}
            });
    }
}

struct Application {
    control: OrbitControl,
    camera: Camera,
    scene: Scene,
    frame_input_generator: FrameInputGenerator,
}

impl Application {
    fn new(frame_input_generator: FrameInputGenerator, viewport: Viewport) -> Self {
        const DEFAULT_DISTANCE: f32 = 300.0;
        const DEFAULT_FOV: Degrees = degrees(22.5);
        const DEFAULT_TARGET: Vec3 = vec3(0.0, 0.0, 0.0);

        let camera = Camera::new_perspective(
            viewport,
            vec3(0.3, -1.0, 1.0).normalize_to(DEFAULT_DISTANCE),
            DEFAULT_TARGET,
            Vec3::unit_z(),
            DEFAULT_FOV,
            0.1,
            10000.0,
        );
        let control = OrbitControl::new(DEFAULT_TARGET, 1.0, 1000.0);
        let scene = Scene::default();

        Self {
            control,
            camera,
            scene,
            frame_input_generator,
        }
    }

    fn handle_redraw(&mut self, context: &WindowedContext) {
        let mut frame_input = self.frame_input_generator.generate(context);

        self.control
            .handle_events(&mut self.camera, &mut frame_input.events);
        self.camera.set_viewport(frame_input.viewport);
        self.scene
            .handle_events(&mut self.camera, &mut frame_input.events);

        let screen = frame_input.screen();
        self.scene.render(&self.camera, &screen);
    }

    fn handle_window_event(
        &mut self,
        event: &WindowEvent,
        context: &WindowedContext,
        control_flow: &mut ControlFlow,
    ) {
        self.frame_input_generator.handle_winit_window_event(event);
        match event {
            WindowEvent::Resized(physical_size) => {
                context.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                context.resize(**new_inner_size);
            }
            WindowEvent::CloseRequested => {
                control_flow.set_exit();
            }
            _ => (),
        }
    }

    fn handle_model_update(&mut self, model_update: ModelUpdate, context: &WindowedContext) {
        match model_update {
            Ok(model) => self.scene = Scene::from_mesh_model(&model, context),
            Err(err) => eprintln!("Error:{:?}", Report::from(err.to_owned())),
        }
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
    fn from_mesh_model(model: &MeshModel, context: &Context) -> Scene {
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
            if let three_d::Event::MouseMotion { button, delta, .. } = event {
                if *button == Some(MouseButton::Right) {
                    let right = camera.right_direction().normalize();
                    let up = right.cross(camera.view_direction());
                    let translation = -delta.0 as f32 * right + delta.1 as f32 * up;
                    let speed = 0.001 * camera.position().magnitude();

                    camera.translate(&(speed * translation));
                }
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
