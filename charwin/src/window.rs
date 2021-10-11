#[cfg(not(target_family = "wasm"))]
pub mod opengl_window;
#[cfg(target_family = "wasm")]
pub mod webgl_window;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use crate::input::{Key, MouseButton};
use crate::platform::{Context, Window};
use charmath::linear::vector::{Vec2, Vec2F};
use std::collections::HashMap;

/// Window size states.
#[repr(u8)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WindowSizeMode {
    Windowed = 0,
    Fullscreen = 1,
}

/// Packaged arguments for creating a window.
///
/// Allows arguments to be changed and passed to the create function of a window.
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
    #[cfg_attr(target_family = "wasm", wasm_bindgen(constructor))]
    pub fn new(title: String, width: u32, height: u32, mode: WindowSizeMode) -> Self {
        WindowCreateArgs {
            title,
            width,
            height,
            mode,
        }
    }
}

/// A universal set of window events each platform's events gets translated into.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WindowEvent {
    Position(i32, i32),
    Size(i32, i32),
    Close,
    Focus(bool),
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

/// Takes raw window events and provides useful ways to operate on them.
pub trait EventManager: 'static {
    fn mouse_pos(&self) -> (f64, f64);
    fn key_pressed(&self, k: Key) -> bool;
    fn focused(&self) -> bool;
    fn mouse_pressed(&self, button: MouseButton) -> bool;
    fn scroll_change(&self) -> (f64, f64);
    fn total_scroll(&self) -> (f64, f64);
    fn screen_size_changed(&self) -> ((i32, i32), bool);
    fn process_events(&mut self, events: &Vec<WindowEvent>);

    fn mouse_left_pressed(&self) -> bool {
        self.mouse_pressed(MouseButton::Button1)
    }
    fn mouse_middle_pressed(&self) -> bool {
        self.mouse_pressed(MouseButton::Button2)
    }
    fn mouse_right_pressed(&self) -> bool {
        self.mouse_pressed(MouseButton::Button3)
    }
    /// Returns cursor pos as a value between 0 and where 0 is the left of the window and 1 is the right.
    fn mouse_x(&self) -> f64 {
        self.mouse_pos().0
    }
    /// Returns cursor pos as a value between 0 and 1 where 0 is the bottom of the window and 1 is the top.
    fn mouse_y(&self) -> f64 {
        self.mouse_pos().1
    }
    fn gl_mouse_vec(&self) -> Vec2F {
        Vec2F::new(self.mouse_x() as f32, 1.0 - self.mouse_y() as f32) * 2.0 - 1.0
    }
    fn win_width(&self) -> u32 {
        self.screen_size_changed().0 .0 as u32
    }
    fn win_height(&self) -> u32 {
        self.screen_size_changed().0 .1 as u32
    }
    fn win_aspect(&self) -> f32 {
        self.win_width() as f32 / self.win_height() as f32
    }
}

/// A standard event manager implementation.
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug)]
pub struct DefaultEventManager {
    mouse_pos: (f64, f64),
    keys: HashMap<Key, bool>,
    mouse_buttons: HashMap<MouseButton, bool>,
    win_size: ((i32, i32), bool),
    win_pos: ((i32, i32), bool),
    focused: bool,
    cursor_on_window: bool,
    scroll_diff: (f64, f64),
    total_scroll: (f64, f64),
}
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl DefaultEventManager {
    #[cfg_attr(target_family = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> DefaultEventManager {
        DefaultEventManager {
            mouse_pos: (0.0, 0.0),
            keys: HashMap::new(),
            mouse_buttons: HashMap::new(),
            win_size: ((0, 0), false),
            win_pos: ((0, 0), false),
            focused: true,
            cursor_on_window: true,
            scroll_diff: (0.0, 0.0),
            total_scroll: (0.0, 0.0),
        }
    }
}
impl EventManager for DefaultEventManager {
    fn screen_size_changed(&self) -> ((i32, i32), bool) {
        self.win_size
    }
    fn mouse_pos(&self) -> (f64, f64) {
        self.mouse_pos
    }
    fn scroll_change(&self) -> (f64, f64) {
        self.scroll_diff
    }
    fn total_scroll(&self) -> (f64, f64) {
        self.total_scroll
    }
    fn focused(&self) -> bool {
        self.focused
    }
    fn key_pressed(&self, k: Key) -> bool {
        if self.keys.contains_key(&k) {
            self.keys[&k]
        } else {
            false
        }
    }
    fn mouse_pressed(&self, button: MouseButton) -> bool {
        if self.mouse_buttons.contains_key(&button) {
            self.mouse_buttons[&button]
        } else {
            false
        }
    }
    fn process_events(&mut self, events: &Vec<WindowEvent>) {
        // crate::platform::dbg_log(&format!("Self: {:?}", self));
        self.scroll_diff = (0.0, 0.0);
        self.win_pos.1 = false;
        self.win_size.1 = false;
        let mut mouse_changed = false;
        let mut mouse_pos = (0.0, 0.0);
        for event in events {
            match event {
                WindowEvent::Size(w, h) => {
                    if self.win_size.0 != (*w, *h) {
                        self.win_size = ((*w, *h), true);
                    }
                }
                WindowEvent::CursorPosition(x, y) => {
                    mouse_pos = (*x, *y);
                    mouse_changed = true;
                }
                WindowEvent::Position(x, y) => {
                    self.win_pos = ((*x, *y), true);
                }
                WindowEvent::Focus(focused) => {
                    self.focused = *focused;
                }
                WindowEvent::KeyUp(key, _) => {
                    self.keys.insert(*key, false);
                }
                WindowEvent::KeyDown(key, _) => {
                    self.keys.insert(*key, true);
                }
                WindowEvent::CursorEnter(cursor_on_window) => {
                    self.cursor_on_window = *cursor_on_window;
                }
                WindowEvent::MouseButtonUp(btn) => {
                    self.mouse_buttons.insert(*btn, false);
                }
                WindowEvent::MouseButtonDown(btn) => {
                    self.mouse_buttons.insert(*btn, true);
                }
                WindowEvent::Scroll(x, y) => {
                    self.scroll_diff = (*x, *y);
                    self.total_scroll.0 += *x;
                    self.total_scroll.1 += *y;
                }
                _ => {}
            }
        }
        if mouse_changed && self.win_size.0 .0 > 0 && self.win_size.0 .1 > 0 {
            self.mouse_pos = (
                mouse_pos.0 / self.win_size.0 .0 as f64,
                mouse_pos.1 / self.win_size.0 .1 as f64,
            );
        }
    }
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl DefaultEventManager {
    #[wasm_bindgen(js_name = mouseX)]
    pub fn wmouse_x(&self) -> f64 {
        self.mouse_x()
    }
    #[wasm_bindgen(js_name = mouseY)]
    pub fn wmouse_y(&self) -> f64 {
        self.mouse_y()
    }
    #[wasm_bindgen(js_name = keyPressed)]
    pub fn wkey_pressed(&self, k: Key) -> bool {
        self.key_pressed(k)
    }
    #[wasm_bindgen(js_name = focused)]
    pub fn wfocused(&self) -> bool {
        self.focused()
    }
    #[wasm_bindgen(js_name = mousePressed)]
    pub fn wmouse_pressed(&self, m: MouseButton) -> bool {
        self.mouse_pressed(m)
    }
    #[wasm_bindgen(js_name = scrollChangeX)]
    pub fn wscroll_change_x(&self) -> f64 {
        self.scroll_change().0
    }
    #[wasm_bindgen(js_name = scrollChangeY)]
    pub fn wscroll_change_y(&self) -> f64 {
        self.scroll_change().1
    }
    #[wasm_bindgen(js_name = totalScrollX)]
    pub fn wtotal_scroll_x(&self) -> f64 {
        self.total_scroll().0
    }
    #[wasm_bindgen(js_name = totalScrollY)]
    pub fn wtotal_scroll_y(&self) -> f64 {
        self.total_scroll().1
    }
    #[wasm_bindgen(js_name = processEvents)]
    pub fn wprocess_event_set(&mut self, set: crate::window::webgl_window::WebWindowEventSet) {
        self.process_events(&set.get_events())
    }
    #[wasm_bindgen(js_name = winSizeChanged)]
    pub fn wwin_size_changed(&self) -> bool {
        self.screen_size_changed().1
    }
    #[wasm_bindgen(js_name = winSize)]
    pub fn wwin_size(&self) -> Vec2F {
        Vec2F::new(self.win_width() as f32, self.win_height() as f32)
    }
    #[wasm_bindgen(js_name = winAspect)]
    pub fn wwin_aspect(&self) -> f32 {
        self.win_aspect()
    }
    #[wasm_bindgen(js_name = glMousePos)]
    pub fn wgl_mouse_pos(&self) -> Vec2F {
        self.gl_mouse_vec()
    }
}

/// A common set of functions for each platform.
pub trait AbstractWindow {
    fn set_fullscreen(&mut self);
    fn set_windowed(&mut self);
    fn set_title(&mut self, name: &str);
    fn set_size(&mut self, sz: (i32, i32));
    fn should_close(&mut self) -> bool;
    fn poll_events(&mut self);
    fn get_events(&mut self) -> Vec<WindowEvent>;
    fn swap_buffers(&mut self);
    fn close(&mut self);
    fn get_size(&self) -> (i32, i32);
    fn get_pos(&self) -> (i32, i32);
    fn get_gl_context(&mut self) -> Context;

    fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.get_gl_context().clear_color(r, g, b, a);
    }
    fn clear(&mut self, mask: &[GlClearMask]) {
        self.get_gl_context().clear(mask);
    }
    fn set_resolution(&mut self, res: (i32, i32)) {
        self.get_gl_context()
            .viewport(0, 0, res.0 as u32, res.1 as u32);
    }
    fn get_width(&self) -> i32 {
        self.get_size().0
    }
    fn get_height(&self) -> i32 {
        self.get_size().1
    }
}

/// Managing window creation.
pub trait AbstractWindowFactory {
    fn create(args: &WindowCreateArgs) -> Self;
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct VertexAttrib(pub u32, pub u32, pub usize, pub usize);
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl VertexAttrib {
    #[wasm_bindgen(constructor)]
    pub fn wnew(index: i32, size: i32, step: i32, offset: i32) -> Self {
        VertexAttrib(index as u32, size as u32, step as usize, offset as usize)
    }
}

#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlDrawMode {
    Triangles = 0b1,
    Points = 0b10,
    LineStrip = 0b100,
    LineLoop = 0b1000,
    Lines = 0b10000,
    TriangleStrip = 0b100000,
    TriangleFan = 0b1000000,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlBufferType {
    ArrayBuffer = 0b1,
    AtomicCounterBuffer = 0b10,
    CopyReadBuffer = 0b100,
    CopyWriteBuffer = 0b1000,
    DispatchIndirectBuffer = 0b10000,
    DrawIndirectBuffer = 0b100000,
    ElementArrayBuffer = 0b1000000,
    PixelPackBuffer = 0b10000000,
    PixelUnpackBuffer = 0b100000000,
    QueryBuffer = 0b1000000000,
    ShaderStorageBuffer = 0b10000000000,
    TextureBuffer = 0b100000000000,
    TransformFeedbackBuffer = 0b1000000000000,
    UniformBuffer = 0b10000000000000,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlStorageMode {
    Static = 0b1,
    Dynamic = 0b10,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlShaderType {
    Vertex = 0b1,
    Fragment = 0b10,
    TessControl = 0b100,
    TessEvaluation = 0b1000,
    Geometry = 0b10000,
    Compute = 0b100000,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlClearMask {
    Color = 0b1,
    Depth = 0b10,
    Accum = 0b100,
    Stencil = 0b1000,
}
#[repr(i64)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlFeature {
    DepthBuffer = 0b1,
}

#[allow(drop_bounds)]
pub trait GlBindable: Sized + Drop {
    fn bind(&self);
    fn unbind(&self);
}
pub trait GlShaderLoc: Sized {}
pub trait GlBuffer: GlBindable {
    fn new(w: &Window, tp: GlBufferType) -> Self;
    fn buffer_data(&self, size: usize, data: *const f32, mode: GlStorageMode);
    fn buffer_sub_data(&self, start: usize, size: usize, data: *const f32);
    fn get_buffer_sub_data(&self, start: usize, size: usize, recv: *mut f32);
    fn get_type(&self) -> GlBufferType;
    fn delete(&mut self);
}
pub trait GlVertexArray: GlBindable {
    fn new(w: &Window) -> Self;
    fn attrib_ptr(&self, v: &VertexAttrib);
    fn remove_attrib_ptr(&self, v: &VertexAttrib);
    fn delete(&mut self);
}
pub trait GlProgram: GlBindable {
    type ShaderLoc: GlShaderLoc;
    type Shader: GlShader;
    fn new(w: &Window) -> Self;
    fn draw_arrays(&self, mode: GlDrawMode, start: i32, len: i32);
    fn shader_loc(&self, name: &str) -> Self::ShaderLoc;
    fn attach_shader(&self, shader: &Self::Shader);
    fn link_program(&self);
    fn get_link_status(&self) -> Option<String>;
    fn uniform_4f(&self, loc: &Self::ShaderLoc, v: (f32, f32, f32, f32));
    fn uniform_3f(&self, loc: &Self::ShaderLoc, v: (f32, f32, f32));
    fn uniform_2f(&self, loc: &Self::ShaderLoc, v: (f32, f32));
    fn uniform_1f(&self, loc: &Self::ShaderLoc, v: f32);
    fn uniform_4i(&self, loc: &Self::ShaderLoc, v: (i32, i32, i32, i32));
    fn uniform_3i(&self, loc: &Self::ShaderLoc, v: (i32, i32, i32));
    fn uniform_2i(&self, loc: &Self::ShaderLoc, v: (i32, i32));
    fn uniform_1i(&self, loc: &Self::ShaderLoc, v: i32);
    fn uniform_mat4f(&self, loc: &Self::ShaderLoc, v: &[f32]);
    fn uniform_mat3f(&self, loc: &Self::ShaderLoc, v: &[f32]);
    fn uniform_mat2f(&self, loc: &Self::ShaderLoc, v: &[f32]);
    fn delete(&mut self);
}
#[allow(drop_bounds)]
pub trait GlShader: Sized + Drop {
    fn new(w: &Window, st: GlShaderType) -> Self;
    fn shader_source(&self, src: &str);
    fn compile(&self);
    fn get_compile_status(&self) -> Option<String>;
    fn get_type(&self) -> GlShaderType;
    fn delete(&mut self);

    fn from_source(w: &Window, st: GlShaderType, src: &str) -> Result<Self, String> {
        let ret = Self::new(w, st);
        ret.shader_source(src);
        ret.compile();
        if let Some(err) = ret.get_compile_status() {
            Err(err)
        } else {
            Ok(ret)
        }
    }
}
pub trait GlContext: Sized {
    fn new(w: &mut Window) -> Self;

    fn clear(&self, mask: &[GlClearMask]);
    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32);
    fn viewport(&self, x: i32, y: i32, w: u32, h: u32);
    fn enable(&mut self, feature: GlFeature);
    fn disable(&mut self, feature: GlFeature);
    fn get_enabled_features(&self) -> Vec<GlFeature>;

    fn enable_features(&mut self, features: &[GlFeature]) {
        for i in 0..features.len() {
            self.enable(features[i]);
        }
    }
    fn disable_features(&mut self, features: &[GlFeature]) {
        for i in 0..features.len() {
            self.disable(features[i]);
        }
    }
}
