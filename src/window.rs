use fj_interop::model::Model;
use fj_viewer::{
    InputEvent, NormalizedScreenPosition, RendererInitError, Screen, ScreenSize, Viewer,
};
use futures::executor::block_on;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};

use crate::model;

pub struct Window {
    window: winit::window::Window,
    event_loop: EventLoop<Result<Model, model::Error>>,
}

impl Window {
    pub fn new() -> Result<Self, Error> {
        let event_loop = EventLoopBuilder::with_user_event().build();
        let window = WindowBuilder::new()
            .with_title("Concavum customizer")
            .with_decorations(false)
            .with_transparent(false)
            .build(&event_loop)?;

        Ok(Self { window, event_loop })
    }

    pub fn event_loop_proxy(&self) -> EventLoopProxy<Result<Model, model::Error>> {
        self.event_loop.create_proxy()
    }

    pub fn run_event_loop(self) -> Result<(), Error> {
        let mut viewer = block_on(Viewer::new(&self))?;

        let mut held_mouse_button = None;
        let mut new_size = None;
        let mut stop_drawing = false;

        let Window { window, event_loop } = self;

        event_loop.run(move |event, _, control_flow| {
            let input_event = input_event(
                &event,
                &window.inner_size(),
                &held_mouse_button,
                viewer.cursor(),
            );
            if let Some(input_event) = input_event {
                viewer.handle_input_event(input_event);
            }

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(virtual_key_code),
                                    ..
                                },
                            ..
                        },
                    ..
                } => match virtual_key_code {
                    VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                    VirtualKeyCode::Key1 => {
                        viewer.toggle_draw_model();
                    }
                    VirtualKeyCode::Key2 => {
                        viewer.toggle_draw_mesh();
                    }
                    _ => {}
                },
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    new_size = Some(ScreenSize {
                        width: size.width,
                        height: size.height,
                    });
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => match state {
                    ElementState::Pressed => {
                        held_mouse_button = Some(button);
                        viewer.add_focus_point();
                    }
                    ElementState::Released => {
                        held_mouse_button = None;
                        viewer.remove_focus_point();
                    }
                },
                Event::WindowEvent {
                    event: WindowEvent::MouseWheel { .. },
                    ..
                } => viewer.add_focus_point(),
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    // Only do a screen resize once per frame. This protects against
                    // spurious resize events that cause issues with the renderer.
                    if let Some(size) = new_size.take() {
                        stop_drawing = size.width == 0 || size.height == 0;
                        if !stop_drawing {
                            viewer.handle_screen_resize(size);
                        }
                    }

                    if !stop_drawing {
                        viewer.draw();
                    }
                }
                Event::UserEvent(model) => match model {
                    Ok(model) => viewer.handle_model_update(model),
                    Err(err) => println!("{:#?}", err),
                },
                _ => {}
            }
        });
    }
}

impl Screen for Window {
    type Window = winit::window::Window;

    fn size(&self) -> ScreenSize {
        let size = &self.window.inner_size();

        ScreenSize {
            width: size.width,
            height: size.height,
        }
    }

    fn window(&self) -> &winit::window::Window {
        &self.window
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error initializing window
    #[error("Error initializing window")]
    WindowInit(#[from] winit::error::OsError),

    /// Error initializing graphics
    #[error("Error initializing graphics")]
    GraphicsInit(#[from] RendererInitError),
}

fn input_event<T>(
    event: &Event<T>,
    screen_size: &PhysicalSize<u32>,
    held_mouse_button: &Option<MouseButton>,
    previous_cursor: &mut Option<NormalizedScreenPosition>,
) -> Option<InputEvent> {
    match event {
        Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } => {
            let width = screen_size.width as f64;
            let height = screen_size.height as f64;
            let aspect_ratio = width / height;

            // Cursor position in normalized coordinates (-1 to +1) with
            // aspect ratio taken into account.
            let current = NormalizedScreenPosition {
                x: position.x / width * 2. - 1.,
                y: -(position.y / height * 2. - 1.) / aspect_ratio,
            };
            let event = match (*previous_cursor, held_mouse_button) {
                (Some(previous), Some(button)) => match button {
                    MouseButton::Left => {
                        let diff_x = current.x - previous.x;
                        let diff_y = current.y - previous.y;
                        let angle_x = -diff_y * ROTATION_SENSITIVITY;
                        let angle_y = diff_x * ROTATION_SENSITIVITY;

                        Some(InputEvent::Rotation { angle_x, angle_y })
                    }
                    MouseButton::Right => Some(InputEvent::Translation { previous, current }),
                    _ => None,
                },
                _ => None,
            };
            *previous_cursor = Some(current);
            event
        }
        Event::WindowEvent {
            event: WindowEvent::MouseWheel { delta, .. },
            ..
        } => {
            let delta = match delta {
                MouseScrollDelta::LineDelta(_, y) => f64::from(*y) * ZOOM_FACTOR_LINE,
                MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => y * ZOOM_FACTOR_PIXEL,
            };

            Some(InputEvent::Zoom(delta))
        }
        _ => None,
    }
}

/// Affects the speed of zoom movement given a scroll wheel input in lines.
///
/// Smaller values will move the camera less with the same input.
/// Larger values will move the camera more with the same input.
const ZOOM_FACTOR_LINE: f64 = 0.075;

/// Affects the speed of zoom movement given a scroll wheel input in pixels.
///
/// Smaller values will move the camera less with the same input.
/// Larger values will move the camera more with the same input.
const ZOOM_FACTOR_PIXEL: f64 = 0.005;

/// Affects the speed of rotation given a change in normalized screen position [-1, 1]
///
/// Smaller values will move the camera less with the same input.
/// Larger values will move the camera more with the same input.
const ROTATION_SENSITIVITY: f64 = 5.;
