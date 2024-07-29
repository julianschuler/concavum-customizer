//! This file provides the entry point and setup for compiling the customizer to WASM.

#![allow(special_module_name)]
#[cfg(target_arch = "wasm32")]
mod main;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// The entry point of the WASM module.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    main::main().expect("executing main should never fail");

    Ok(())
}
