use glfw::{
    Context as GlContext, Glfw, Window as GlfwWindow, WindowEvent as GlWindowEvent,
    WindowMode as GlWindowMode,
};

extern crate gl;
use crate::input::{Key, MouseButton};
use crate::state::{FrameManager, State};
use crate::window::{
    AbstractWindow, AbstractWindowFactory, EventManager, WindowCreateArgs, WindowEvent,
    WindowSizeMode,
};
use gl::types::{GLenum, GLfloat};

pub struct NativeGlWindow {
    glfw: Glfw,
    window: GlfwWindow,
    gl_events: std::sync::mpsc::Receiver<(f64, GlWindowEvent)>,
    events: Vec<WindowEvent>,
    clear_mask: GLenum,
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
    }
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
    fn get_size(&self) -> (i32, i32) {
        self.window.get_size()
    }
    fn get_pos(&self) -> (i32, i32) {
        self.window.get_pos()
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
        glfw_window.set_all_polling(true);
        glfw_window.make_current();
        gl::load_with(|s| glfw.get_proc_address_raw(s));
        NativeGlWindow {
            glfw,
            window: glfw_window,
            gl_events: glfw_events,
            clear_mask: gl::COLOR_BUFFER_BIT,
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
