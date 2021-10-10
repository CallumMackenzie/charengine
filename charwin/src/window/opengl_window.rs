use glfw::{
    Context as GlfwContext, Glfw, Window as GlfwWindow, WindowEvent as GlWindowEvent,
    WindowMode as GlWindowMode,
};
use std::ffi::CString;
use std::ptr;
extern crate gl;
use crate::data::buffers::VertexAttrib;
use crate::input::{Key, MouseButton};
use crate::platform::{Context, Window};
use crate::state::{FrameManager, State};
use crate::window::{
    AbstractWindow, AbstractWindowFactory, EventManager, GlBindable, GlBuffer, GlBufferType,
    GlClearMask, GlContext, GlDrawMode, GlProgram, GlShader, GlShaderLoc, GlShaderType,
    GlStorageMode, GlVertexArray, WindowCreateArgs, WindowEvent, WindowSizeMode,
};
use gl::types::{GLbitfield, GLchar, GLenum, GLint, GLintptr, GLsizei, GLsizeiptr, GLuint, GLvoid};

pub struct NativeGlWindow {
    glfw: Glfw,
    window: GlfwWindow,
    gl_events: std::sync::mpsc::Receiver<(f64, GlWindowEvent)>,
    events: Vec<WindowEvent>,
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
        let mut fm = FrameManager::new(60f64);
        let mut state_res = state.initialize(&mut self, &mut manager);
        self.get_gl_errors();
        if state_res == 0 {
            println!("State initialized.");
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
        println!("State destroyed.");
        self.get_gl_errors();
    }
    fn get_gl_errors(&self) {
        unsafe {
            let mut err = gl::GetError();
            while err != gl::NO_ERROR {
                match err {
                    gl::INVALID_ENUM => {
                        panic!("OpenGL: GL_INVALID_ENUM (0x0500)");
                    }
                    gl::INVALID_VALUE => {
                        panic!("OpenGL: GL_INVALID_VALUE (0x0501)");
                    }
                    gl::INVALID_OPERATION => {
                        panic!("OpenGL: GL_INVALID_OPERATION (0x0502)");
                    }
                    gl::STACK_OVERFLOW => {
                        panic!("OpenGL: GL_STACK_OVERFLOW (0x0503)");
                    }
                    gl::STACK_UNDERFLOW => {
                        panic!("OpenGL: GL_STACK_UNDERFLOW (0x0504)");
                    }
                    gl::OUT_OF_MEMORY => {
                        panic!("OpenGL: GL_OUT_OF_MEMORY (0x0505)");
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

pub struct NativeGlContext {}
impl GlContext for NativeGlContext {
    fn new(_: &mut Window) -> Self {
        Self {}
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
}

pub struct NativeGlBuffer {
    pub vbo: GLuint,
    buff_type: GlBufferType,
    gl_buff_type: GLenum,
}
impl Drop for NativeGlBuffer {
    fn drop(&mut self) {
        self.delete();
    }
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
    fn delete(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.vbo);
        }
    }
}

pub struct NativeGlVertexArray {
    pub vao: GLuint,
}
impl Drop for NativeGlVertexArray {
    fn drop(&mut self) {
        self.delete();
    }
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
    fn delete(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
        self.vao = gl::NONE;
    }
}

pub struct NativeGlShader {
    pub shader: GLuint,
    pub stype: GlShaderType,
}
impl Drop for NativeGlShader {
    fn drop(&mut self) {
        self.delete();
    }
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
    fn delete(&mut self) {
        unsafe {
            gl::DeleteShader(self.shader);
        }
        self.shader = gl::NONE;
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
impl Drop for NativeGlProgram {
    fn drop(&mut self) {
        self.delete();
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
    fn delete(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
        self.program = gl::NONE;
    }
}
