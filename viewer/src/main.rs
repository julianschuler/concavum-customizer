//! This file starts the viewer by creating a new window and running the render loop.

use three_d::WindowError;
use viewer::Window;

fn main() -> Result<(), WindowError> {
    let window = Window::try_new()?;
    window.run_render_loop();

    Ok(())
}
