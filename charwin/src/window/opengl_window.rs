use glfw::{
    Context as GlContext, Glfw, Window as GlfwWindow, WindowEvent as GlWindowEvent,
    WindowMode as GlWindowMode,
};

extern crate gl;
use crate::input::{Key, MouseButton};
use crate::window::{
    AbstractWindow, AbstractWindowFactory, WindowCreateArgs, WindowEvent, WindowSizeMode,
};
use gl::types::{GLenum, GLfloat};

pub struct NativeGlWindow {
    glfw: Glfw,
    window: GlfwWindow,
    events: std::sync::mpsc::Receiver<(f64, GlWindowEvent)>,
    clear_mask: GLenum,
}

impl AbstractWindow for NativeGlWindow {
    fn set_fullscreen(&mut self) {
        unimplemented!();
    }
    fn set_windowed(&mut self) {
        unimplemented!();
    }
    fn set_title(&mut self, name: &str) {
        self.window.set_title(name);
    }
    fn set_size(&mut self, w: u32, h: u32) {
        self.window.set_size(w as i32, h as i32);
    }
    fn should_close(&mut self) -> bool {
        self.window.should_close()
    }
    fn poll_events(&mut self) {
        self.glfw.poll_events();
    }
    fn get_events(&mut self) -> Vec<WindowEvent> {
        let mut events: Vec<WindowEvent> = Vec::new();
        for (_, event) in glfw::flush_messages(&self.events) {
            events.push(gl_event_to_window_event(event));
        }
        events
    }
    fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }
    fn close(&mut self) {
        self.window.set_should_close(true);
    }
    fn set_clear_colour(&mut self, r: f64, g: f64, b: f64, a: f64) {
        unsafe {
            gl::ClearColor(r as GLfloat, g as GLfloat, b as GLfloat, a as GLfloat);
        }
    }
    fn clear(&mut self) {
        unsafe {
            gl::Clear(self.clear_mask);
        }
    }
}

impl AbstractWindowFactory for NativeGlWindow {
    fn create(args: &WindowCreateArgs) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let (mut glfw_window, glfw_events) = glfw
            .create_window(args.width, args.height, &args.title, GlWindowMode::Windowed)
            .expect("Failed to create a GLFW window.");
        if args.mode == WindowSizeMode::Fullscreen {
            glfw.with_primary_monitor(|_: &mut _, m: Option<&glfw::Monitor>| {
                let monitor = m.unwrap();
                let mode: glfw::VidMode = monitor.get_video_mode().unwrap();
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
        glfw_window.set_key_polling(true);
        glfw_window.make_current();
        gl::load_with(|s| glfw.get_proc_address_raw(s));
        NativeGlWindow {
            glfw,
            window: glfw_window,
            events: glfw_events,
            clear_mask: gl::COLOR_BUFFER_BIT,
        }
    }
}

fn gl_event_to_window_event(gl_event: GlWindowEvent) -> WindowEvent {
    match gl_event {
        GlWindowEvent::Pos(x, y) => WindowEvent::Position(x, y),
        GlWindowEvent::Size(x, y) => WindowEvent::Size(x, y),
        GlWindowEvent::Close => WindowEvent::Close,
        GlWindowEvent::Focus(is) => WindowEvent::Focus(is),
        GlWindowEvent::FramebufferSize(x, y) => WindowEvent::FrameBufferSize(x, y),
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
    }
}
