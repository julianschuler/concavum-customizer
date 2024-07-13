use std::sync::mpsc::Receiver;

use color_eyre::Report;
use config::Config;
use gui::{Gui, SceneUpdate, SceneUpdater};
use three_d::{
    degrees, vec3, window, Camera, Context, Degrees, FrameInput, FrameOutput, InnerSpace,
    MouseButton, OrbitControl, Vec3, WindowError, WindowSettings,
};
use winit::event_loop::{EventLoop, EventLoopProxy};

use crate::{assets::Assets, scene::Scene};

/// An application window.
pub struct Window {
    inner: window::Window,
    event_loop_proxy: EventLoopProxy<()>,
}

impl Window {
    /// Creates a new window.
    ///
    /// Returns a [`WindowError`] the window could not be created.
    pub fn try_new() -> Result<Self, WindowError> {
        let event_loop = EventLoop::new();
        let event_loop_proxy = event_loop.create_proxy();
        let inner = window::Window::from_event_loop(
            WindowSettings {
                title: "Concavum customizer".to_owned(),
                ..Default::default()
            },
            event_loop,
        )?;

        Ok(Self {
            inner,
            event_loop_proxy,
        })
    }

    /// Runs the render loop using the given initial config.
    ///
    /// This is blocking until the window is closed.
    pub fn run_render_loop(self, initial_config: Config) {
        let mut application = Application::new(&self.inner, self.event_loop_proxy, initial_config);

        self.inner.render_loop(move |frame_input| {
            application.handle_events(frame_input);
            FrameOutput::default()
        });
    }
}

/// An application rendering an interactive scene and GUI.
struct Application {
    control: OrbitControl,
    camera: Camera,
    scene: Scene,
    assets: Assets,
    receiver: Receiver<SceneUpdate>,
    gui: Gui,
    show_spinner: bool,
}

impl Application {
    /// Creates a new Application given a window, event loop proxy and initial config.
    fn new(
        window: &window::Window,
        event_loop_proxy: EventLoopProxy<()>,
        initial_config: Config,
    ) -> Self {
        const DEFAULT_DISTANCE: f32 = 600.0;
        const DEFAULT_FOV: Degrees = degrees(22.5);
        const DEFAULT_TARGET: Vec3 = vec3(0.0, 0.0, 0.0);

        let context = window.gl();
        let camera = Camera::new_perspective(
            window.viewport(),
            vec3(0.3, -1.0, 1.0).normalize_to(DEFAULT_DISTANCE),
            DEFAULT_TARGET,
            Vec3::unit_z(),
            DEFAULT_FOV,
            0.1,
            10000.0,
        );
        let control = OrbitControl::new(DEFAULT_TARGET, 50.0, 5000.0);
        let scene = Scene::default();
        let assets = Assets::new();

        let (updater, receiver) = SceneUpdater::from_event_loop_proxy(event_loop_proxy);
        let gui = Gui::from_config(&context, initial_config, updater);

        Self {
            control,
            camera,
            scene,
            assets,
            receiver,
            gui,
            show_spinner: false,
        }
    }

    /// Handles events for the given frame input.
    fn handle_events(&mut self, mut frame_input: FrameInput) {
        self.gui.update(&mut frame_input, self.show_spinner);

        self.camera.set_viewport(frame_input.viewport);
        self.control
            .handle_events(&mut self.camera, &mut frame_input.events);

        // Allow translating the camera sideways when holding right mouse button
        for event in &mut frame_input.events {
            if let three_d::Event::MouseMotion { button, delta, .. } = event {
                if *button == Some(MouseButton::Right) {
                    let right = self.camera.right_direction().normalize();
                    let up = right.cross(self.camera.view_direction());
                    let translation = -delta.0 * right + delta.1 * up;
                    let speed = 0.001 * self.camera.position().magnitude();

                    self.camera.translate(&(speed * translation));
                }
            }
        }

        if let Ok(scene_update) = self.receiver.try_recv() {
            self.handle_scene_update(&frame_input.context, scene_update);
        }

        // Render scene and GUI
        let screen = frame_input.screen();
        self.scene.render(&self.camera, &screen);
        self.gui.render(&screen)
    }

    /// Handles a scene update for the given context.
    fn handle_scene_update(&mut self, context: &Context, scene_update: SceneUpdate) {
        self.show_spinner = matches!(
            &scene_update,
            SceneUpdate::Model(_) | SceneUpdate::Preview(_)
        );

        match scene_update {
            SceneUpdate::New(model, meshes) => {
                self.scene = Scene::from_model(context, model, &self.assets);
                self.scene.update_objects(context, &meshes);
            }
            SceneUpdate::Model(model) => {
                self.scene = Scene::from_model(context, model, &self.assets);
            }
            SceneUpdate::Preview(mesh) => {
                self.scene.update_preview(context, &mesh);
            }
            SceneUpdate::Meshes(meshes) => self.scene.update_objects(context, &meshes),
            SceneUpdate::Error(err) => eprintln!("Error:{:?}", Report::from(err.clone())),
        }
    }
}
