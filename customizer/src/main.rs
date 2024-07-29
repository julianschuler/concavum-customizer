//! This file starts the customizer by creating a new window and running the render loop.

use viewer::{Window, WindowError};

/// The main function of the customizer.
///
/// # Errors
///
/// Returns a `WindowError` if the window could not be created.
pub fn main() -> Result<(), WindowError> {
    let window = Window::try_new()?;
    window.run_render_loop();

    Ok(())
}
