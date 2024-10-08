//! This file provides the entry point and setup for compiling the customizer to WASM.

use viewer::Window;
use wasm_bindgen::prelude::*;

/// The entry point of the WASM module.
///
/// # Panics
///
/// Panics if the window could not be created.
#[allow(clippy::missing_errors_doc)]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = Window::try_new().expect("creating the window should never fail");
    window.run_render_loop();

    Ok(())
}
