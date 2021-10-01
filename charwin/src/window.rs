#[cfg(not(target_family = "wasm"))]
pub mod opengl_window;
#[cfg(target_family = "wasm")]
pub mod webgl_window;

#[cfg(not(target_family = "wasm"))]
pub type PlatformWindow = crate::window::opengl_window::NativeGlWindow;
#[cfg(target_family = "wasm")]
pub type PlatformWindow = crate::window::webgl_window::WebGlWindow;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use crate::input::{Key, MouseButton};

#[repr(u8)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WindowSizeMode {
    Windowed = 0,
    Fullscreen = 1,
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug)]
pub struct WindowCreateArgs {
    title: String,
    pub width: u32,
    pub height: u32,
    pub mode: WindowSizeMode,
}
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl WindowCreateArgs {
    #[cfg(target_family = "wasm")]
    #[wasm_bindgen(js_name = toString)]
    pub fn wto_string(&self) -> String {
        format!("{:?}", self).into()
    }
    pub fn new(title: String, width: u32, height: u32, mode: WindowSizeMode) -> Self {
        WindowCreateArgs {
            title,
            width,
            height,
            mode,
        }
    }
}

#[derive(Debug)]
pub enum WindowEvent {
    Position(i32, i32),
    Size(i32, i32),
    Close,
    Focus(bool),
    FrameBufferSize(i32, i32),
    MouseButtonUp(MouseButton),
    MouseButtonDown(MouseButton),
    MouseButtonHeld(MouseButton),
    CursorPosition(f64, f64),
    CursorEnter(bool),
    Scroll(f64, f64),
    KeyUp(Key, i32),
    KeyDown(Key, i32),
    KeyHeld(Key, i32),
    None,
}

pub trait AbstractWindow {
    fn set_fullscreen(&mut self);
    fn set_windowed(&mut self);
    fn set_title(&mut self, name: &str);
    fn set_size(&mut self, w: u32, h: u32);
    fn should_close(&mut self) -> bool;
    fn poll_events(&mut self);
    fn get_events(&mut self) -> Vec<WindowEvent>;
    fn swap_buffers(&mut self);
    fn close(&mut self);
    fn set_clear_colour(&mut self, r: f64, g: f64, b: f64, a: f64);
    fn clear(&mut self);
}

pub trait AbstractWindowFactory {
    fn create(args: &WindowCreateArgs) -> Self;
}
