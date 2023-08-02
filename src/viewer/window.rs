use std::sync::Arc;

use color_eyre::Report;
use three_d::{
    degrees, vec3, Camera, ClearState, Color, CpuMaterial, DirectionalLight, Event::UserEvent,
    FrameOutput, Gm, InstancedMesh, Instances, Mesh, OrbitControl, PhysicalMaterial, WindowError,
    WindowSettings,
};
use winit::event_loop::{EventLoopBuilder, EventLoopProxy};

use crate::model::Error;

use super::model::MeshModel;

pub type ModelUpdate = Result<MeshModel, Arc<Error>>;

pub struct Window {
    window: three_d::Window<ModelUpdate>,
    event_loop_proxy: EventLoopProxy<ModelUpdate>,
    objects: Vec<Gm<Mesh, PhysicalMaterial>>,
    instanced_objects: Vec<Gm<InstancedMesh, PhysicalMaterial>>,
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
            objects: Vec::new(),
            instanced_objects: Vec::new(),
        })
    }

    pub fn event_loop_proxy(&self) -> EventLoopProxy<ModelUpdate> {
        self.event_loop_proxy.clone()
    }

    pub fn run_render_loop(mut self) {
        let context = self.window.gl();

        let mut camera = Camera::new_perspective(
            self.window.viewport(),
            vec3(60.00, 50.0, 60.0), // camera position
            vec3(0.0, 0.0, 0.0),     // camera target
            vec3(0.0, 0.0, 0.1),     // camera up
            degrees(45.0),
            0.1,
            1000.0,
        );
        let mut control = OrbitControl::new(vec3(0.0, 0.0, 0.0), 1.0, 1000.0);

        let light1 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, -0.5, -0.5));
        let light2 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, 0.5, 0.5));

        self.window.render_loop(move |mut frame_input| {
            control.handle_events(&mut camera, &mut frame_input.events);

            let screen = frame_input.screen();

            for event in frame_input.events.iter() {
                if let UserEvent(model_update) = event {
                    match model_update {
                        Ok(model) => {
                            self.objects.clear();
                            self.instanced_objects.clear();

                            for object in model.objects.iter() {
                                let material = &CpuMaterial {
                                    albedo: object.color,
                                    ..Default::default()
                                };
                                let material = PhysicalMaterial::new(&context, &material);

                                if let Some(transformations) = &object.transformations {
                                    let mesh = InstancedMesh::new(
                                        &context,
                                        &Instances {
                                            transformations: transformations.to_owned(),
                                            ..Default::default()
                                        },
                                        &object.mesh,
                                    );
                                    self.instanced_objects.push(Gm::new(mesh, material));
                                } else {
                                    let mesh = Mesh::new(&context, &object.mesh);
                                    self.objects.push(Gm::new(mesh, material));
                                }
                            }
                        }
                        Err(err) => eprintln!("Error:{:?}", Report::from(err.to_owned())),
                    }
                }
            }

            screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

            screen.render(&camera, &self.objects, &[&light1, &light2]);
            screen.render(&camera, &self.instanced_objects, &[&light1, &light2]);

            FrameOutput::default()
        })
    }
}
