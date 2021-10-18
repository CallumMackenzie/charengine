#[cfg(not(target_family = "wasm"))]
pub mod opengl_window;
#[cfg(target_family = "wasm")]
pub mod webgl_window;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use crate::data::GPUTexture;
use crate::input::{Key, MouseButton};
use crate::platform::{Context, Window};
use charmath::linear::vector::{Vec2, Vec2F};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    fn cursor_on_window(&self) -> bool;

    /// Whether the left mouse button is pressed
    ///
    /// Example code
    /// let man = DefaultEventManager::new();
    /// man.process_events(&vec![WindowEvent::MouseButtonDown(MouseButton::Button1)]);
    /// if man.mouse_left_pressed() {
    ///     println!("LMB pressed");
    /// }
    fn mouse_left_pressed(&self) -> bool {
        self.mouse_pressed(MouseButton::Button1)
    }
    /// Whether the middle mouse button is pressed
    ///
    /// Example code
    /// let man = DefaultEventManager::new();
    /// man.process_events(&vec![WindowEvent::MouseButtonDown(MouseButton::Button2)]);
    /// if man.mouse_middle_pressed() {
    ///     println!("MMB pressed");
    /// }
    fn mouse_middle_pressed(&self) -> bool {
        self.mouse_pressed(MouseButton::Button2)
    }
    /// Whether the right mouse button is pressed
    ///
    /// Example code
    /// let man = DefaultEventManager::new();
    /// man.process_events(&vec![WindowEvent::MouseButtonDown(MouseButton::Button3)]);
    /// if man.mouse_right_pressed() {
    ///     println!("RMB pressed");
    /// }
    fn mouse_right_pressed(&self) -> bool {
        self.mouse_pressed(MouseButton::Button3)
    }
    /// Returns normalized cursor x coordinate.
    ///
    /// Example code
    /// println!("Mouse x pos: {}", man.mouse_x());
    fn mouse_x(&self) -> f64 {
        self.mouse_pos().0
    }
    /// Returns normalized cursor x coordinate.
    ///
    /// Example code
    /// println!("Mouse y pos: {}", man.mouse_y());
    fn mouse_y(&self) -> f64 {
        self.mouse_pos().1
    }
    /// Returns the cursor pos in normalized OpenGl coordinates
    fn gl_mouse_vec(&self) -> Vec2F {
        Vec2F::new(self.mouse_x() as f32, 1.0 - self.mouse_y() as f32) * 2.0 - 1.0
    }
    /// Returns the width of the window
    fn win_width(&self) -> u32 {
        self.screen_size_changed().0 .0 as u32
    }
    /// Returns the height of the window
    fn win_height(&self) -> u32 {
        self.screen_size_changed().0 .1 as u32
    }
    /// Returns a tuple with the width and height of the window
    fn win_size(&self) -> (u32, u32) {
        (self.win_width(), self.win_height())
    }
    /// Returns the window width divided by the window height
    fn win_aspect_x(&self) -> f32 {
        self.win_width() as f32 / self.win_height() as f32
    }
    /// Returns the window height divided by the window width
    fn win_aspect_y(&self) -> f32 {
        self.win_height() as f32 / self.win_width() as f32
    }
    /// Returns the window x and y aspects as a tuple
    fn win_aspects(&self) -> (f32, f32) {
        (self.win_aspect_x(), self.win_aspect_y())
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
    fn cursor_on_window(&self) -> bool {
        self.cursor_on_window
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
    #[wasm_bindgen(js_name = winAspectX)]
    pub fn wwin_aspect_x(&self) -> f32 {
        self.win_aspect_x()
    }
    #[wasm_bindgen(js_name = glMousePos)]
    pub fn wgl_mouse_pos(&self) -> Vec2F {
        self.gl_mouse_vec()
    }
}

/// A common set of window functions for each platform.
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
    fn load_texture_rgba(&mut self, path: &str, mips: Option<u32>) -> Arc<Mutex<GPUTexture>>;

    fn clear_colour(&mut self) {
        self.clear(&[GlClearMask::Color]);
    }
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
    Triangles = 0x1,
    Points = 0x2,
    LineStrip = 0x4,
    LineLoop = 0x8,
    Lines = 0x10,
    TriangleStrip = 0x20,
    TriangleFan = 0x40,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlBufferType {
    ArrayBuffer = 0x1,
    AtomicCounterBuffer = 0x2,
    CopyReadBuffer = 0x4,
    CopyWriteBuffer = 0x8,
    DispatchIndirectBuffer = 0x10,
    DrawIndirectBuffer = 0x20,
    ElementArrayBuffer = 0x40,
    PixelPackBuffer = 0x80,
    PixelUnpackBuffer = 0x100,
    QueryBuffer = 0x200,
    ShaderStorageBuffer = 0x400,
    TextureBuffer = 0x800,
    TransformFeedbackBuffer = 0x1000,
    UniformBuffer = 0x2000,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlStorageMode {
    Static = 0x1,
    Dynamic = 0x2,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlShaderType {
    Vertex = 0x1,
    Fragment = 0x2,
    TessControl = 0x4,
    TessEvaluation = 0x8,
    Geometry = 0x10,
    Compute = 0x20,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlClearMask {
    Color = 0x1,
    Depth = 0x2,
    Accum = 0x4,
    Stencil = 0x8,
}

#[repr(i64)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlFeature {
    AlphaTest = 0x1,
    AutoNormal = 0x2,
    Blend = 0x4,
    ColorLogicOp = 0x8,
    ColorMaterial = 0x10,
    ColorSum = 0x20,
    ColorTable = 0x40,
    CullFace = 0x80,
    DepthTest = 0x100,
    Dither = 0x200,
    LineSmooth = 0x400,
    PolygonOffsetFill = 0x800,
    PolygonOffsetLine = 0x1000,
    PolygonOffsetPoint = 0x2000,
    PolygonSmooth = 0x4000,
    SampleAlphaToCoverage = 0x8000,
    SampleAlphaToOne = 0x10000,
    SampleCoverage = 0x20000,
    ScissorTest = 0x40000,
    StencilTest = 0x80000,
    Texture1D = 0x100000,
    Texture2D = 0x200000,
    Texture3D = 0x400000,
    TextureCubeMap = 0x800000,
    VertexProgramPointSize = 0x1000000,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlTextureType {
    Texture2D = 0x1,
    ProxyTexture2D = 0x2,
    Texture1DArray = 0x4,
    ProxyTexture1DArray = 0x8,
    TextureRectangle = 0x10,
    ProxyTextureRectangle = 0x20,
    CubeMapPositiveX = 0x40,
    CubeMapNegativeX = 0x80,
    CubeMapPositiveY = 0x100,
    CubeMapNegativeY = 0x200,
    CubeMapPositiveZ = 0x400,
    CubeMapNegativeZ = 0x800,
    ProxyCubeMap = 0x1000,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlInternalTextureFormat {
    Red,
    RG,
    RGB,
    RGBA,
    R8,
    R8SNorm,
    R16,
    R16SNorm,
    RG8,
    RG8SNorm,
    RG16,
    RG16SNorm,
    R3G3B2,
    RGB4,
    RGB5,
    RGB8,
    RGB8SNorm,
    RGB10,
    RGB12,
    RGB16Snorm,
    RGBA2,
    RGBA4,
    RGB5A1,
    RGBA8,
    RGBA8SNorm,
    RGB10A2,
    RGB10A2UI,
    RGBA12,
    RGBA16,
    SRGB8,
    SRGB8Alpha8,
    R16F,
    RG16F,
    RGB16F,
    RGBA16F,
    R32F,
    RG32F,
    RGB32F,
    RGBA32F,
    R11FG11FB10F,
    RGB9E5,
    R8I,
    R8UI,
    R16I,
    R16UI,
    R32I,
    R32UI,
    RG8I,
    RG8UI,
    RG16I,
    RG16UI,
    RG32I,
    RG32UI,
    RGB8I,
    RGB8UI,
    RGB16I,
    RGB16UI,
    RGB32I,
    RGB32UI,
    RGBA8I,
    RGBA8UI,
    RGBA16I,
    RGBA16UI,
    RGBA32I,
    RGBA32UI,
    CompressedRed,
    CompressedRG,
    CompressedRGB,
    CompressedRGBA,
    CompressedSRGB,
    CompressedSRGBAlpha,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlImagePixelFormat {
    Red,
    RG,
    RGB,
    BGR,
    RGBA,
    BGRA,
    RedInt,
    RGInt,
    RGBInt,
    BGRInt,
    RGBAInt,
    BGRAInt,
    StencilIndex,
    DepthComponent,
    DepthStencil,
}
#[repr(i32)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlImagePixelType {
    UnsignedByte,
    Byte,
    UnsignedShort,
    Short,
    UnsignedInt,
    Int,
    HalfFloat,
    Float,
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
}
pub trait GlVertexArray: GlBindable {
    fn new(w: &Window) -> Self;
    fn attrib_ptr(&self, v: &VertexAttrib);
    fn remove_attrib_ptr(&self, v: &VertexAttrib);
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
}
#[allow(drop_bounds)]
pub trait GlShader: Sized + Drop {
    fn new(w: &Window, st: GlShaderType) -> Self;
    fn shader_source(&self, src: &str);
    fn compile(&self);
    fn get_compile_status(&self) -> Option<String>;
    fn get_type(&self) -> GlShaderType;

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
pub trait GlTexture2D: GlBindable {
    fn new(w: &mut Window) -> Self;
	/// For mipmap levels, None means use default levels, Some(x) means generate x mipmap levels.
    fn set_texture(
        &self,
        tex: *const u8,
        width: u32,
        height: u32,
        internal_fmt: GlInternalTextureFormat,
        img_fmt: GlImagePixelFormat,
        px_type: GlImagePixelType,
        mipmaps: Option<u32>,
        pixel_byte_size: usize,
    );
    fn set_slot(&mut self, slot: u32);
}
pub trait GlContext: Sized {
    fn new(w: &mut Window) -> Self;

    fn clear(&self, mask: &[GlClearMask]);
    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32);
    fn viewport(&self, x: i32, y: i32, w: u32, h: u32);
    fn enable(&mut self, feature: GlFeature);
    fn disable(&mut self, feature: GlFeature);
    fn get_enabled_features(&self) -> Vec<GlFeature>;
	fn default_depth_func(&self);

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
