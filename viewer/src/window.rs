use std::sync::mpsc::Receiver;

use gui::{Gui, Settings, Update, Updater};
use three_d::{
    degrees, vec3, window, Camera, Context, Degrees, FrameInput, FrameOutput, InnerSpace,
    MouseButton, OrbitControl, Vec3, Viewport, WindowError, WindowSettings,
};

use crate::{assets::Assets, scene::Scene};

/// An application window.
pub struct Window {
    inner: window::Window,
}

impl Window {
    /// Creates a new window.
    ///
    /// # Errors
    ///
    /// Returns a [`WindowError`] if the window could not be created.
    pub fn try_new() -> Result<Self, WindowError> {
        let inner = window::Window::new(WindowSettings {
            title: "Concavum customizer".to_owned(),
            ..Default::default()
        })?;

        Ok(Self { inner })
    }

    /// Runs the render loop. This is blocking until the window is closed.
    pub fn run_render_loop(self) {
        let mut application = Application::new(&self.inner);

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
    receiver: Receiver<Update>,
    gui: Gui,
    is_reloading: bool,
}

impl Application {
    /// Creates a new Application given a window and event loop proxy.
    fn new(window: &window::Window) -> Self {
        const DEFAULT_DISTANCE: f32 = 800.0;
        const DEFAULT_FOV: Degrees = degrees(22.5);
        const DEFAULT_TARGET: Vec3 = vec3(0.0, 0.0, 0.0);

        let context = window.gl();
        let camera = Camera::new_perspective(
            window.viewport(),
            vec3(0.3, -1.0, 1.0).normalize_to(DEFAULT_DISTANCE),
            DEFAULT_TARGET,
            Vec3::unit_z(),
            DEFAULT_FOV,
            1.0,
            4000.0,
        );
        let control = OrbitControl::new(DEFAULT_TARGET, 50.0, 2000.0);
        let assets = Assets::new();
        let scene = Scene::from_settings(&context, Settings::default(), &assets);

        let (updater, receiver) = Updater::new();
        let gui = Gui::new(&context, updater);

        Self {
            control,
            camera,
            scene,
            assets,
            receiver,
            gui,
            is_reloading: false,
        }
    }

    /// Handles events for the given frame input.
    fn handle_events(&mut self, mut frame_input: FrameInput) {
        self.gui.update(&mut frame_input, self.is_reloading);

        #[allow(clippy::cast_possible_truncation)]
        let viewport = Viewport {
            x: (Gui::SIDE_PANEL_WIDTH * frame_input.device_pixel_ratio) as i32,
            y: 0,
            #[allow(clippy::cast_sign_loss)]
            width: frame_input.viewport.width
                - (Gui::SIDE_PANEL_WIDTH * frame_input.device_pixel_ratio) as u32,
            height: frame_input.viewport.height,
        };
        self.camera.set_viewport(viewport);
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
        self.gui.render(&screen);
    }

    /// Handles a scene update.
    fn handle_scene_update(&mut self, context: &Context, scene_update: Update) {
        if !matches!(&scene_update, Update::DisplaySettings(_)) {
            self.is_reloading = matches!(&scene_update, Update::Settings(_) | Update::Preview(_));
        }

        match scene_update {
            Update::New(model, meshes) => {
                self.scene = Scene::from_settings(context, model, &self.assets);
                self.scene.update_keyboard(context, &meshes);
            }
            Update::Settings(settings) => {
                self.scene = Scene::from_settings(context, settings, &self.assets);
            }
            Update::DisplaySettings(display_settings) => {
                self.scene.update_display_settings(display_settings);
            }
            Update::Preview(mesh) => {
                self.scene.update_preview(context, &mesh);
            }
            Update::Meshes(meshes) => self.scene.update_keyboard(context, &meshes),
        }
    }
}
