#[cfg(not(target_family = "wasm"))]
pub type Window = crate::window::opengl_window::NativeGlWindow;
#[cfg(not(target_family = "wasm"))]
pub type Context = crate::window::opengl_window::NativeGlContext;
#[cfg(not(target_family = "wasm"))]
pub type Shader = crate::window::opengl_window::NativeGlShader;
#[cfg(not(target_family = "wasm"))]
pub type Buffer = crate::window::opengl_window::NativeGlBuffer;
#[cfg(not(target_family = "wasm"))]
pub type VertexArray = crate::window::opengl_window::NativeGlVertexArray;
#[cfg(not(target_family = "wasm"))]
pub type Program = crate::window::opengl_window::NativeGlProgram;
#[cfg(not(target_family = "wasm"))]
pub type Texture2D = crate::window::opengl_window::NativeGlTexture2D;

#[cfg(target_family = "wasm")]
pub type Window = crate::window::webgl_window::WebGlWindow;
#[cfg(target_family = "wasm")]
pub type Context = crate::window::webgl_window::WebGlContext;
#[cfg(target_family = "wasm")]
pub type Shader = crate::window::webgl_window::WebGlShader;
#[cfg(target_family = "wasm")]
pub type Buffer = crate::window::webgl_window::WebGlBuffer;
#[cfg(target_family = "wasm")]
pub type VertexArray = crate::window::webgl_window::WebGlVertexArray;
#[cfg(target_family = "wasm")]
pub type Program = crate::window::webgl_window::WebGlProgram;
#[cfg(target_family = "wasm")]
pub type Texture2D = crate::window::webgl_window::WebGlTexture2D;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn js_log_string(a: &str);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn js_err_string(a: &str);
}

#[cfg(target_family = "wasm")]
pub fn dbg_log(s: &str) {
    js_log_string(s);
}
#[cfg(not(target_family = "wasm"))]
pub fn dbg_log(s: &str) {
    println!("{}", s);
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! char_panic {
    ($($arg : tt) *) => {
        panic!($($arg)*);
    };
}
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! char_panic {
    ($($arg : tt) *) => {
        crate::platform::js_err_string(&format!($($arg)*));
        panic!();
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! cw_panic {
    ($($arg : tt) *) => {
        panic!($($arg)*);
    };
}
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! cw_panic {
    ($($arg : tt) *) => {
        charwin::platform::js_err_string(&format!($($arg)*));
        panic!();
    };
}

#[macro_export]
macro_rules! cw_println {
    ($($arg : tt) *) => {
		charwin::platform::dbg_log(&format!($($arg)*));
    };
}
