/// The window for the current plaform.
#[cfg(not(target_family = "wasm"))]
pub type Window = crate::window::opengl_window::NativeGlWindow;
#[cfg(target_family = "wasm")]
pub type Window = crate::window::webgl_window::WebGlWindow;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log_string(a: &str);
}

/// Platform-agnostic logging.
#[cfg(target_family = "wasm")]
pub fn dbg_log(s: &str) {
    js_log_string(s);
}
#[cfg(not(target_family = "wasm"))]
pub fn dbg_log(s: &str) {
    println!("{}", s);
}
