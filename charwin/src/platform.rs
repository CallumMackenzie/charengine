#[cfg(not(target_family = "wasm"))]
pub type Window = crate::window::opengl_window::NativeGlWindow;
#[cfg(target_family = "wasm")]
pub type Window = crate::window::webgl_window::WebGlWindow;
