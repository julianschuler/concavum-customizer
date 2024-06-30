use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
};

use color_eyre::Report;
use three_d::{
    degrees,
    egui::{Align2, Area, Spinner},
    vec3, window, Camera, Context, Degrees, FrameInput, FrameOutput, InnerSpace, MouseButton,
    OrbitControl, Vec3, WindowError, WindowSettings, GUI,
};
use winit::event_loop::{EventLoop, EventLoopProxy};

use crate::{
    config::Error,
    viewer::{assets::Assets, model::Model, scene::Scene},
};

/// A reload event.
pub enum ReloadEvent {
    Started,
    Updated(Model),
    Finished(Model),
    Error(Arc<Error>),
}

/// A model updater allowing to update a model at runtime.
#[derive(Clone)]
pub struct ModelUpdater {
    sender: Sender<ReloadEvent>,
    event_loop_proxy: EventLoopProxy<()>,
}

impl ModelUpdater {
    pub fn send_event(&self, event: ReloadEvent) {
        let _ = self.sender.send(event);
        let _ = self.event_loop_proxy.send_event(());
    }
}

/// An application window with a model updater.
pub struct Window {
    inner: window::Window,
    updater: ModelUpdater,
    receiver: Receiver<ReloadEvent>,
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

        let (sender, receiver) = channel();

        let updater = ModelUpdater {
            sender,
            event_loop_proxy,
        };

        Ok(Self {
            inner,
            updater,
            receiver,
        })
    }

    /// Returns the model updater.
    pub fn model_updater(&self) -> ModelUpdater {
        self.updater.clone()
    }

    /// Runs the render loop.
    ///
    /// This is blocking until the window is closed.
    pub fn run_render_loop(self) {
        let mut application = Application::new(&self.inner, self.receiver);

        self.inner.render_loop(move |frame_input| {
            application.handle_events(frame_input);
            FrameOutput::default()
        });
    }
}

/// An application rendering an interactive Scene.
struct Application {
    control: OrbitControl,
    camera: Camera,
    scene: Scene,
    gui: GUI,
    receiver: Receiver<ReloadEvent>,
    assets: Assets,
    show_spinner: bool,
}

impl Application {
    /// Creates a new Application from a viewport and a receiver with reload events.
    fn new(window: &window::Window, receiver: Receiver<ReloadEvent>) -> Self {
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
        let gui = GUI::new(&context);
        let assets = Assets::new();

        Self {
            control,
            camera,
            scene,
            gui,
            receiver,
            assets,
            show_spinner: false,
        }
    }

    /// Handles events for the given frame input.
    fn handle_events(&mut self, mut frame_input: FrameInput) {
        self.gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                if self.show_spinner {
                    Area::new("area")
                        .anchor(Align2::RIGHT_BOTTOM, [-15.0, -15.0])
                        .show(gui_context, |ui| ui.add(Spinner::new().size(32.0)));
                }
            },
        );

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

        if let Ok(reload_event) = self.receiver.try_recv() {
            self.handle_reload_event(&frame_input.context, reload_event);
        }

        // Render scene and GUI
        let screen = frame_input.screen();
        self.scene.render(&self.camera, &screen);
        screen
            .write(|| self.gui.render())
            .expect("rendering the gui should never fail");
    }

    /// Handles a reload event for the given context.
    fn handle_reload_event(&mut self, context: &Context, reload_event: ReloadEvent) {
        self.show_spinner = matches!(
            &reload_event,
            ReloadEvent::Started | ReloadEvent::Updated(_)
        );

        match reload_event {
            ReloadEvent::Started => {}
            ReloadEvent::Updated(model) => {
                self.scene = Scene::from_model(context, model, &self.assets, true);
            }
            ReloadEvent::Finished(model) => {
                self.scene = Scene::from_model(context, model, &self.assets, false);
            }
            ReloadEvent::Error(err) => eprintln!("Error:{:?}", Report::from(err.clone())),
        }
    }
}
