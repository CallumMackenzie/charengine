use glfw::{
    Context as GlfwContext, Glfw, Window as GlfwWindow, WindowEvent as GlWindowEvent,
    WindowMode as GlWindowMode,
};
use std::ffi::CString;
use std::ptr;
extern crate gl;
use crate::data::{CPUBuffer, DynamicImageColorable, GPUTexture};
use crate::input::{Key, MouseButton};
use crate::platform::{Context, Window};
use crate::state::{FrameManager, State};
use crate::window::*;
use gl::types::{GLbitfield, GLchar, GLenum, GLint, GLintptr, GLsizei, GLsizeiptr, GLuint, GLvoid};
use image::io::Reader as ImageReader;
use image::DynamicImage;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub struct NativeGlWindow {
    glfw: Glfw,
    window: GlfwWindow,
    gl_events: Receiver<(f64, GlWindowEvent)>,
    events: Vec<WindowEvent>,
    image_load_threads: HashMap<
        u32,
        (
            Arc<Mutex<GPUTexture>>,
            JoinHandle<DynamicImage>,
            Receiver<()>,
			Option<u32>,
        ),
    >,
    image_thread_count: u32,
}

impl NativeGlWindow {
    pub fn render_loop<S: State, E: EventManager>(mut self, mut state: S, mut manager: E) {
        {
            self.poll_events();
            let mut init_events = self.get_events();
            let size = self.get_size();
            let pos = self.get_pos();
            init_events.push(WindowEvent::Size(size.0, size.1));
            init_events.push(WindowEvent::Position(pos.0, pos.1));
            manager.process_events(&init_events);
        }
        let mut fm = FrameManager::new(None);
        let mut state_res = state.initialize(&mut self, &mut manager);
        self.get_gl_errors();
        if state_res == 0 {
            loop {
                if fm.next_frame_ready() {
                    self.poll_events();
                    manager.process_events(&self.get_events());
                    state_res = state.update(&mut self, &mut manager, fm.get_delta());
                    if state_res != 0 || self.should_close() {
                        break;
                    }
                }
            }
        }
        state.destroy(&mut self, &mut manager, state_res);
        self.get_gl_errors();
    }
    fn get_gl_errors(&self) {
        unsafe {
            let mut err = gl::GetError();
            while err != gl::NO_ERROR {
                match err {
                    gl::INVALID_ENUM => {
                        panic!("OpenGL: GL_INVALID_ENUM (0x0500). Enumeration parameter is not legal for function.");
                    }
                    gl::INVALID_VALUE => {
                        panic!("OpenGL: GL_INVALID_VALUE (0x0501). Value parameter is not legal for function.");
                    }
                    gl::INVALID_OPERATION => {
                        panic!(
                            "OpenGL: GL_INVALID_OPERATION (0x0502). State is invalid for function."
                        );
                    }
                    gl::STACK_OVERFLOW => {
                        panic!("OpenGL: GL_STACK_OVERFLOW (0x0503). Stack pushing operation cannot be done due to stack size.");
                    }
                    gl::STACK_UNDERFLOW => {
                        panic!("OpenGL: GL_STACK_UNDERFLOW (0x0504). Stack pop operation cannot be done due to stack size.");
                    }
                    gl::OUT_OF_MEMORY => {
                        panic!(
                            "OpenGL: GL_OUT_OF_MEMORY (0x0505). Cannot allocate more heap memory."
                        );
                    }
                    gl::INVALID_FRAMEBUFFER_OPERATION => {
                        panic!("OpenGL: GL_INVALID_FRAMEBUFFER_OPERATION (0x0506)");
                    }
                    gl::CONTEXT_LOST => {
                        panic!("OpenGL: GL_CONTEXT_LOST (0x0507)");
                    }
                    _ => {
                        println!("OpenGL: Error code: {}.", err);
                    }
                }
                err = gl::GetError();
            }
        }
    }
    fn poll_image_threads(&mut self) {
        let mut completed_threads =
            Vec::<(bool, u32)>::with_capacity(self.image_load_threads.len());
        for (index, thread) in self.image_load_threads.iter() {
            match thread.2.try_recv() {
                Ok(_) => {
                    completed_threads.push((true, *index));
                }
                Err(TryRecvError::Disconnected) => {
                    eprintln!("Image thread {} disconnected.", index);
                    completed_threads.push((false, *index));
                }
                _ => {}
            }
        }
        for (success, index) in completed_threads {
            if success {
                let (tex, thread, _, mips) = self.image_load_threads.remove(&index).unwrap();
                match thread.join() {
                    Ok(data) => {
                        tex.lock().unwrap().set_data_mips(&data, mips);
                    }
                    Err(e) => {
                        eprintln!("Image thread {} error: {:?}", index, e);
                    }
                }
            } else {
                self.image_load_threads.remove(&index);
            }
        }
    }
}

impl AbstractWindow for NativeGlWindow {
    fn get_gl_context(&mut self) -> Context {
        Context::new(self)
    }
    fn set_fullscreen(&mut self) {
        unimplemented!();
    }
    fn set_windowed(&mut self) {
        unimplemented!();
    }
    fn set_title(&mut self, name: &str) {
        self.window.set_title(name);
    }
    fn set_size(&mut self, sz: (i32, i32)) {
        self.window.set_size(sz.0, sz.1);
    }
    fn should_close(&mut self) -> bool {
        self.window.should_close()
    }
    fn poll_events(&mut self) {
        self.glfw.poll_events();
        self.events.clear();
        for (_, gl_event) in glfw::flush_messages(&self.gl_events) {
            if let Some(event) = gl_event_to_window_event(gl_event) {
                self.events.push(event);
            }
        }
        self.poll_image_threads();
    }
    fn get_events(&mut self) -> Vec<WindowEvent> {
        let mut events = Vec::new();
        for i in 0..self.events.len() {
            events.push(self.events[i]);
        }
        events
    }
    fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }
    fn close(&mut self) {
        self.window.set_should_close(true);
    }
    fn get_size(&self) -> (i32, i32) {
        self.window.get_size()
    }
    fn get_pos(&self) -> (i32, i32) {
        self.window.get_pos()
    }
    fn load_texture_rgba(&mut self, path: &str, mips: Option<u32>) -> Arc<Mutex<GPUTexture>> {
        let tex = Arc::new(Mutex::new(
            DynamicImage::solid_color([0xff, 0x80, 0xff, 0xff]).to_gpu_buffer(self),
        ));
        let (sender, reciever) = channel();
        let image_src: String = path.into();
        let handle = thread::spawn(move || {
            let ret = ImageReader::open(&image_src)
                .unwrap_or_else(|e| {
                    panic!("Could not load image \"{}\": {:?}.", image_src, e);
                })
                .decode()
                .unwrap_or_else(|e| {
                    panic!("Could not decode image \"{}\": {:?}.", image_src, e);
                });
            sender.send(()).unwrap_or_else(|e| {
                panic!("Could not send message: {:?}", e);
            });
            ret
        });
        self.image_load_threads.insert(
            self.image_thread_count,
            (Arc::clone(&tex), handle, reciever, mips),
        );
        self.image_thread_count += 1;
        tex
    }
}

impl AbstractWindowFactory for NativeGlWindow {
    fn create(args: &WindowCreateArgs) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("GLFW: Initialization failed.");
        let (mut glfw_window, glfw_events) = glfw
            .create_window(args.width, args.height, &args.title, GlWindowMode::Windowed)
            .expect("GLFW: Failed to create a window.");
        if args.mode == WindowSizeMode::Fullscreen {
            glfw.with_primary_monitor(|_: &mut _, m: Option<&glfw::Monitor>| {
                let monitor = m.expect("GLFW: Could not get primary monitor.");
                let mode: glfw::VidMode = monitor
                    .get_video_mode()
                    .expect("GLFW: Could not get primary monitor video mode.");
                glfw_window.set_monitor(
                    glfw::WindowMode::FullScreen(&monitor),
                    0,
                    0,
                    mode.width,
                    mode.height,
                    Some(mode.refresh_rate),
                );
            });
        }
        glfw_window.make_current();
        glfw_window.set_all_polling(true);
        gl::load_with(|s| glfw.get_proc_address_raw(s));
        NativeGlWindow {
            glfw,
            window: glfw_window,
            gl_events: glfw_events,
            events: Vec::new(),
            image_load_threads: HashMap::new(),
            image_thread_count: 0,
        }
    }
}

fn gl_event_to_window_event(gl_event: GlWindowEvent) -> Option<WindowEvent> {
    let event = match gl_event {
        GlWindowEvent::Pos(x, y) => WindowEvent::Position(x, y),
        GlWindowEvent::Size(x, y) => WindowEvent::Size(x, y),
        GlWindowEvent::Close => WindowEvent::Close,
        GlWindowEvent::Focus(is) => WindowEvent::Focus(is),
        GlWindowEvent::MouseButton(button, action, _mods) => match action {
            glfw::Action::Release => {
                WindowEvent::MouseButtonUp(MouseButton::from_i32(button as i32))
            }
            glfw::Action::Press => {
                WindowEvent::MouseButtonDown(MouseButton::from_i32(button as i32))
            }
            glfw::Action::Repeat => {
                WindowEvent::MouseButtonHeld(MouseButton::from_i32(button as i32))
            }
        },
        GlWindowEvent::CursorPos(x, y) => WindowEvent::CursorPosition(x, y),
        GlWindowEvent::CursorEnter(is) => WindowEvent::CursorEnter(is),
        GlWindowEvent::Scroll(x, y) => WindowEvent::Scroll(x, y),
        GlWindowEvent::Key(key, code, action, _mods) => match action {
            glfw::Action::Release => WindowEvent::KeyUp(Key::from_i32(key as i32), code),
            glfw::Action::Press => WindowEvent::KeyDown(Key::from_i32(key as i32), code),
            glfw::Action::Repeat => WindowEvent::KeyHeld(Key::from_i32(key as i32), code),
        },
        _ => WindowEvent::None,
    };
    if event == WindowEvent::None {
        None
    } else {
        Some(event)
    }
}

pub struct NativeGlContext {
    features: HashSet<GlFeature>,
}
impl NativeGlContext {
    fn gl_feature(f: &GlFeature) -> GLenum {
        use GlFeature::*;
        match f {
            // AlphaTest => gl::ALPHA_TEST,
            // AutoNormal => gl::AUTO_NORMAL,
            Blend => gl::BLEND,
            ColorLogicOp => gl::COLOR_LOGIC_OP,
            // ColorMaterial => gl::COLOR_MATERIAL,
            // ColorSum => gl::COLOR_SUM,
            // ColorTable => gl::COLOR_TABLE,
            CullFace => gl::CULL_FACE,
            DepthTest => gl::DEPTH_TEST,
            Dither => gl::DITHER,
            LineSmooth => gl::LINE_SMOOTH,
            PolygonOffsetFill => gl::POLYGON_OFFSET_FILL,
            PolygonOffsetLine => gl::POLYGON_OFFSET_LINE,
            PolygonOffsetPoint => gl::POLYGON_OFFSET_POINT,
            PolygonSmooth => gl::POLYGON_SMOOTH,
            SampleAlphaToCoverage => gl::SAMPLE_ALPHA_TO_COVERAGE,
            SampleAlphaToOne => gl::SAMPLE_ALPHA_TO_ONE,
            SampleCoverage => gl::SAMPLE_COVERAGE,
            ScissorTest => gl::SCISSOR_TEST,
            StencilTest => gl::STENCIL_TEST,
            Texture1D => gl::TEXTURE_1D,
            Texture2D => gl::TEXTURE_2D,
            Texture3D => gl::TEXTURE_3D,
            TextureCubeMap => gl::TEXTURE_CUBE_MAP,
            VertexProgramPointSize => gl::VERTEX_PROGRAM_POINT_SIZE,
            _ => {
                panic!("WebGL: GlFeature {:?} not supported for wasm.", f);
            }
        }
    }
}
impl GlContext for NativeGlContext {
    fn new(_: &mut Window) -> Self {
        Self {
            features: HashSet::with_capacity(10),
        }
    }
    fn clear(&self, mask: &[GlClearMask]) {
        use GlClearMask::*;
        let mut glmask: GLbitfield = 0;
        for i in 0..mask.len() {
            glmask |= match mask[i] {
                Color => gl::COLOR_BUFFER_BIT,
                Depth => gl::DEPTH_BUFFER_BIT,
                Stencil => gl::STENCIL_BUFFER_BIT,
                _ => 0,
            }
        }
        unsafe {
            gl::Clear(glmask);
        }
    }
    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
    }
    fn viewport(&self, x: i32, y: i32, w: u32, h: u32) {
        unsafe {
            gl::Viewport(x, y, w as i32, h as i32);
        }
    }
    fn enable(&mut self, feature: GlFeature) {
        self.features.insert(feature);
        unsafe { gl::Enable(Self::gl_feature(&feature)) }
    }
    fn disable(&mut self, feature: GlFeature) {
        self.features.remove(&feature);
        unsafe { gl::Disable(Self::gl_feature(&feature)) }
    }
    fn get_enabled_features(&self) -> Vec<GlFeature> {
        self.features.iter().map(|x| *x).collect()
    }
    fn default_depth_func(&self) {
        unsafe {
            gl::DepthFunc(gl::LEQUAL);
        }
    }
}

pub struct NativeGlBuffer {
    pub vbo: GLuint,
    buff_type: GlBufferType,
    gl_buff_type: GLenum,
}
impl NativeGlBuffer {
    fn buff_type(t: &GlBufferType) -> GLenum {
        use GlBufferType::*;
        match t {
            ArrayBuffer => gl::ARRAY_BUFFER,
            AtomicCounterBuffer => gl::ATOMIC_COUNTER_BUFFER,
            CopyReadBuffer => gl::COPY_READ_BUFFER,
            CopyWriteBuffer => gl::COPY_WRITE_BUFFER,
            DispatchIndirectBuffer => gl::DISPATCH_INDIRECT_BUFFER,
            DrawIndirectBuffer => gl::DRAW_INDIRECT_BUFFER,
            ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER,
            PixelPackBuffer => gl::PIXEL_PACK_BUFFER,
            PixelUnpackBuffer => gl::PIXEL_UNPACK_BUFFER,
            QueryBuffer => gl::QUERY_BUFFER,
            ShaderStorageBuffer => gl::SHADER_STORAGE_BUFFER,
            TextureBuffer => gl::TEXTURE_BUFFER,
            TransformFeedbackBuffer => gl::TRANSFORM_FEEDBACK_BUFFER,
            UniformBuffer => gl::UNIFORM_BUFFER,
        }
    }
    fn storage_mode(t: &GlStorageMode) -> GLenum {
        use GlStorageMode::*;
        match t {
            Static => gl::STATIC_DRAW,
            Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}
impl GlBindable for NativeGlBuffer {
    fn bind(&self) {
        unsafe {
            gl::BindBuffer(Self::buff_type(&self.buff_type), self.vbo);
        }
    }
    fn unbind(&self) {
        unsafe {
            gl::BindBuffer(Self::buff_type(&self.buff_type), gl::NONE);
        }
    }
}
impl GlBuffer for NativeGlBuffer {
    fn new(_: &Window, buff_type: GlBufferType) -> Self {
        let mut v = gl::NONE;
        unsafe {
            gl::GenBuffers(1, &mut v);
        }
        Self {
            vbo: v,
            buff_type,
            gl_buff_type: Self::buff_type(&buff_type),
        }
    }
    fn get_type(&self) -> GlBufferType {
        self.buff_type
    }
    fn buffer_data(&self, size: usize, data: *const f32, mode: GlStorageMode) {
        unsafe {
            gl::BufferData(
                self.gl_buff_type,
                size as isize,
                data as *const GLvoid,
                Self::storage_mode(&mode),
            );
        }
    }
    fn buffer_sub_data(&self, start: usize, size: usize, data: *const f32) {
        unsafe {
            gl::BufferSubData(
                self.gl_buff_type,
                start as GLintptr,
                size as GLsizeiptr,
                data as *const GLvoid,
            );
        }
    }
    fn get_buffer_sub_data(&self, start: usize, size: usize, recv: *mut f32) {
        unsafe {
            gl::GetBufferSubData(
                self.gl_buff_type,
                start as GLintptr,
                size as GLsizeiptr,
                recv as *mut GLvoid,
            );
        }
    }
}
impl Drop for NativeGlBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.vbo);
        }
    }
}

pub struct NativeGlVertexArray {
    pub vao: GLuint,
}
impl GlBindable for NativeGlVertexArray {
    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }
    fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(gl::NONE);
        }
    }
}
impl GlVertexArray for NativeGlVertexArray {
    fn new(_: &Window) -> Self {
        let mut v = gl::NONE;
        unsafe {
            gl::GenVertexArrays(1, &mut v);
        }
        Self { vao: v }
    }
    fn attrib_ptr(&self, a: &VertexAttrib) {
        unsafe {
            gl::VertexAttribPointer(
                (a.0) as GLuint,
                (a.1) as GLint,
                gl::FLOAT,
                gl::FALSE,
                (a.2) as GLsizei,
                ((a.3) as GLuint) as *const GLvoid,
            );
            gl::EnableVertexAttribArray((a.0) as GLuint);
        }
    }
    fn remove_attrib_ptr(&self, a: &VertexAttrib) {
        unsafe {
            gl::DisableVertexAttribArray(a.0 as GLuint);
        }
    }
}
impl Drop for NativeGlVertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}

pub struct NativeGlShader {
    pub shader: GLuint,
    pub stype: GlShaderType,
}
impl NativeGlShader {
    fn shader_type(t: &GlShaderType) -> GLenum {
        use GlShaderType::*;
        match t {
            Vertex => gl::VERTEX_SHADER,
            Fragment => gl::FRAGMENT_SHADER,
            TessControl => gl::TESS_CONTROL_SHADER,
            TessEvaluation => gl::TESS_EVALUATION_SHADER,
            Geometry => gl::GEOMETRY_SHADER,
            Compute => gl::COMPUTE_SHADER,
        }
    }
}
impl GlShader for NativeGlShader {
    fn new(_: &Window, st: GlShaderType) -> Self {
        unsafe {
            Self {
                shader: gl::CreateShader(Self::shader_type(&st)),
                stype: st,
            }
        }
    }
    fn shader_source(&self, src: &str) {
        unsafe {
            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(self.shader, 1, &c_str.as_ptr(), ptr::null());
        }
    }
    fn compile(&self) {
        unsafe {
            gl::CompileShader(self.shader);
        }
    }
    fn get_compile_status(&self) -> Option<String> {
        unsafe {
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(self.shader, gl::COMPILE_STATUS, &mut status);
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(self.shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);
                gl::GetShaderInfoLog(
                    self.shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                Some(
                    format!(
                        "OpenGL: Shader compilation error: {}",
                        std::str::from_utf8(buf.as_slice())
                            .ok()
                            .unwrap_or_else(|| "ShaderInfoLog not valid utf8")
                    )
                    .into(),
                )
            } else {
                None
            }
        }
    }
    fn get_type(&self) -> GlShaderType {
        self.stype
    }
}
impl Drop for NativeGlShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.shader);
        }
    }
}

pub struct NativeGlShaderLoc {
    pub loc: GLint,
}
impl GlShaderLoc for NativeGlShaderLoc {}
pub struct NativeGlProgram {
    pub program: GLuint,
}
impl NativeGlProgram {
    fn draw_mode(m: &GlDrawMode) -> GLenum {
        use GlDrawMode::*;
        match m {
            Triangles => gl::TRIANGLES,
            Points => gl::POINTS,
            LineStrip => gl::LINE_STRIP,
            LineLoop => gl::LINE_LOOP,
            Lines => gl::LINES,
            TriangleStrip => gl::TRIANGLE_STRIP,
            TriangleFan => gl::TRIANGLE_FAN,
        }
    }
}
impl GlBindable for NativeGlProgram {
    fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }
    fn unbind(&self) {
        unsafe {
            gl::UseProgram(gl::NONE);
        }
    }
}
impl GlProgram for NativeGlProgram {
    type ShaderLoc = NativeGlShaderLoc;
    type Shader = NativeGlShader;
    fn new(_: &Window) -> Self {
        Self {
            program: unsafe { gl::CreateProgram() },
        }
    }
    fn draw_arrays(&self, mode: GlDrawMode, start: i32, len: i32) {
        unsafe {
            gl::DrawArrays(Self::draw_mode(&mode), start, len);
        }
    }
    fn shader_loc(&self, name: &str) -> Self::ShaderLoc {
        Self::ShaderLoc {
            loc: unsafe {
                let c_str = CString::new(name.as_bytes()).unwrap();
                gl::GetUniformLocation(self.program, c_str.as_ptr() as *const GLchar)
            },
        }
    }
    fn attach_shader(&self, shader: &Self::Shader) {
        unsafe {
            gl::AttachShader(self.program, shader.shader);
        }
    }
    fn link_program(&self) {
        unsafe {
            gl::LinkProgram(self.program);
        }
    }
    fn get_link_status(&self) -> Option<String> {
        unsafe {
            let mut link_success: GLint = gl::TRUE as GLint;
            gl::GetProgramiv(self.program, gl::LINK_STATUS, &mut link_success);
            if link_success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetProgramiv(self.program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);
                gl::GetProgramInfoLog(
                    self.program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                Some(
                    format!(
                        "OpenGL program creation error: {}",
                        std::str::from_utf8(buf.as_slice())
                            .ok()
                            .unwrap_or_else(|| "ProgramInfoLog not valid utf8")
                    )
                    .into(),
                )
            } else {
                None
            }
        }
    }
    fn uniform_4f(&self, loc: &Self::ShaderLoc, v: (f32, f32, f32, f32)) {
        unsafe {
            gl::Uniform4f(loc.loc, v.0, v.1, v.2, v.3);
        }
    }
    fn uniform_3f(&self, loc: &Self::ShaderLoc, v: (f32, f32, f32)) {
        unsafe {
            gl::Uniform3f(loc.loc, v.0, v.1, v.2);
        }
    }
    fn uniform_2f(&self, loc: &Self::ShaderLoc, v: (f32, f32)) {
        unsafe {
            gl::Uniform2f(loc.loc, v.0, v.1);
        }
    }
    fn uniform_1f(&self, loc: &Self::ShaderLoc, v: f32) {
        unsafe {
            gl::Uniform1f(loc.loc, v);
        }
    }
    fn uniform_4i(&self, loc: &Self::ShaderLoc, v: (i32, i32, i32, i32)) {
        unsafe {
            gl::Uniform4i(loc.loc, v.0, v.1, v.2, v.3);
        }
    }
    fn uniform_3i(&self, loc: &Self::ShaderLoc, v: (i32, i32, i32)) {
        unsafe {
            gl::Uniform3i(loc.loc, v.0, v.1, v.2);
        }
    }
    fn uniform_2i(&self, loc: &Self::ShaderLoc, v: (i32, i32)) {
        unsafe {
            gl::Uniform2i(loc.loc, v.0, v.1);
        }
    }
    fn uniform_1i(&self, loc: &Self::ShaderLoc, v: i32) {
        unsafe {
            gl::Uniform1i(loc.loc, v);
        }
    }
    fn uniform_mat4f(&self, loc: &Self::ShaderLoc, v: &[f32]) {
        unsafe {
            gl::UniformMatrix4fv(loc.loc, 1, 0, &v[0] as *const f32);
        }
    }
    fn uniform_mat3f(&self, loc: &Self::ShaderLoc, v: &[f32]) {
        unsafe {
            gl::UniformMatrix3fv(loc.loc, 1, 0, &v[0] as *const f32);
        }
    }
    fn uniform_mat2f(&self, loc: &Self::ShaderLoc, v: &[f32]) {
        unsafe {
            gl::UniformMatrix2fv(loc.loc, 1, 0, &v[0] as *const f32);
        }
    }
}
impl Drop for NativeGlProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

pub struct NativeGlTexture2D {
    slot: u32,
    tex: GLuint,
}
impl NativeGlTexture2D {
    pub fn gl_texture_type(t: &GlTextureType) -> u32 {
        use GlTextureType::*;
        match t {
            Texture2D => gl::TEXTURE_2D,
            CubeMapPositiveX => gl::TEXTURE_CUBE_MAP_POSITIVE_X,
            CubeMapNegativeX => gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
            CubeMapPositiveY => gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
            CubeMapNegativeY => gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            CubeMapPositiveZ => gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
            CubeMapNegativeZ => gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
            ProxyTexture2D => gl::PROXY_TEXTURE_2D,
            ProxyTexture1DArray => gl::PROXY_TEXTURE_1D_ARRAY,
            Texture1DArray => gl::TEXTURE_1D_ARRAY,
            TextureRectangle => gl::TEXTURE_RECTANGLE,
            ProxyTextureRectangle => gl::PROXY_TEXTURE_RECTANGLE,
            ProxyCubeMap => gl::PROXY_TEXTURE_CUBE_MAP,
        }
    }
    pub fn gl_internal_fmt(f: &GlInternalTextureFormat) -> u32 {
        use GlInternalTextureFormat::*;
        match f {
            Red => gl::RED,
            RG => gl::RG,
            RGB => gl::RGB,
            RGBA => gl::RGBA,
            R8 => gl::R8,
            R8SNorm => gl::R8_SNORM,
            RG8 => gl::RG8,
            RG8SNorm => gl::RG8_SNORM,
            RGB8 => gl::RGB8,
            RGB8SNorm => gl::RGB8_SNORM,
            RGBA4 => gl::RGBA4,
            RGB5A1 => gl::RGB5_A1,
            RGBA8 => gl::RGBA8,
            RGBA8SNorm => gl::RGBA8_SNORM,
            RGB10A2 => gl::RGB10_A2,
            RGB10A2UI => gl::RGB10_A2UI,
            SRGB8 => gl::SRGB8,
            SRGB8Alpha8 => gl::SRGB8_ALPHA8,
            R16F => gl::R16F,
            RG16F => gl::RG16F,
            RGBA16F => gl::RGBA16F,
            R32F => gl::R32F,
            RG32F => gl::RG32F,
            RGBA32F => gl::RGBA32F,
            R11FG11FB10F => gl::R11F_G11F_B10F,
            RGB9E5 => gl::RGB9_E5,
            R8I => gl::R8I,
            R8UI => gl::R8UI,
            R16I => gl::R16I,
            R16UI => gl::R16UI,
            R32I => gl::R32I,
            R32UI => gl::R32UI,
            RG8I => gl::RG8I,
            RG8UI => gl::RG8UI,
            RG16I => gl::RG16I,
            RG16UI => gl::RG16UI,
            RG32I => gl::RG32I,
            RG32UI => gl::RG32UI,
            RGBA8I => gl::RGBA8I,
            RGBA16I => gl::RGBA16I,
            RGBA32I => gl::RGBA32I,
            _ => {
                panic!(
                    "OpenGL: GlInternalTextureFormat {:?} not supported natively.",
                    f
                );
            }
        }
    }
    pub fn gl_img_fmt(f: &GlImagePixelFormat) -> u32 {
        use GlImagePixelFormat::*;
        match f {
            Red => gl::RED,
            RG => gl::RG,
            RGB => gl::RGB,
            RGBA => gl::RGBA,
            RedInt => gl::RED_INTEGER,
            RGInt => gl::RG_INTEGER,
            RGBInt => gl::RGB_INTEGER,
            RGBAInt => gl::RGBA_INTEGER,
            DepthComponent => gl::DEPTH_COMPONENT,
            DepthStencil => gl::DEPTH_STENCIL,
            _ => {
                panic!("OpenGL: GlImagePixelFormat {:?} not supported natively.", f);
            }
        }
    }
    pub fn gl_px_fmt(f: &GlImagePixelType) -> u32 {
        use GlImagePixelType::*;
        match f {
            UnsignedByte => gl::UNSIGNED_BYTE,
            Byte => gl::BYTE,
            UnsignedShort => gl::UNSIGNED_SHORT,
            Short => gl::SHORT,
            UnsignedInt => gl::UNSIGNED_INT,
            Int => gl::INT,
            HalfFloat => gl::HALF_FLOAT,
            Float => gl::FLOAT,
        }
    }
    pub fn set_params(&self, mips: Option<u32>) {
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			if mips == Some(0) {
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
			} else {
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
				gl::GenerateMipmap(gl::TEXTURE_2D);
			}
        }
    }
}
impl GlBindable for NativeGlTexture2D {
    fn bind(&self) {
        unsafe {
            gl::ActiveTexture(self.slot);
            gl::BindTexture(gl::TEXTURE_2D, self.tex)
        }
    }
    fn unbind(&self) {
        unsafe {
            gl::ActiveTexture(self.slot);
            gl::BindTexture(gl::TEXTURE_2D, gl::NONE)
        }
    }
}
impl GlTexture2D for NativeGlTexture2D {
    fn new(_: &mut Window) -> Self {
        let mut tex = gl::NONE;
        unsafe {
            gl::GenTextures(1, &mut tex);
        }
        Self {
            tex,
            slot: gl::TEXTURE0,
        }
    }
    fn set_texture(
        &self,
        tex_ptr: *const u8,
        width: u32,
        height: u32,
        internal_fmt: GlInternalTextureFormat,
        img_fmt: GlImagePixelFormat,
        px_type: GlImagePixelType,
        mipmaps: Option<u32>,
        _px_byte_size: usize,
    ) {
        unsafe {
			let mips = if u32::is_power_of_two(width) && u32::is_power_of_two(height) {
				mipmaps
			} else {
				if mipmaps != None {
					eprintln!("Image improper size ({}x{}) to support specific mipmap level {}.", width, height, mipmaps.unwrap());
				}
				None
			};
            gl::TexImage2D(
                gl::TEXTURE_2D,
                mips.unwrap_or_else(|| 0) as i32,
                Self::gl_internal_fmt(&internal_fmt) as i32,
                width as i32,
                height as i32,
                0,
                Self::gl_img_fmt(&img_fmt),
                Self::gl_px_fmt(&px_type),
                tex_ptr as *const GLvoid,
            );
            self.set_params(mipmaps);
        }
    }
    fn set_slot(&mut self, slot: u32) {
        self.slot = gl::TEXTURE0 + slot;
    }
}
impl Drop for NativeGlTexture2D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.tex);
        }
    }
}
