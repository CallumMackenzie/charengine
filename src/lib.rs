#[cfg(any(test, target_family = "wasm"))]
mod tests {
    use charwin::input::Key;
    use charwin::platform::{dbg_log, Window};
    use charwin::state::State;
    use charwin::window::{
        AbstractWindow, AbstractWindowFactory, DefaultEventManager, EventManager, WindowCreateArgs,
        WindowSizeMode,
    };

    #[cfg(target_family = "wasm")]
    use wasm_bindgen::prelude::*;

    pub struct App {
        ctr: f64,
        swap: f64,
    }
    impl App {
        pub fn new() -> App {
            App {
                ctr: 0.0,
                swap: 0.5,
            }
        }
    }
    impl State for App {
        fn initialize(&mut self, win: &mut Window, _manager: &mut dyn EventManager) -> i32 {
            win.set_clear_colour(0.0, 0.0, 0.0, 1.0);
            0
        }
        fn update(&mut self, win: &mut Window, eng: &mut dyn EventManager, delta: f64) -> i32 {
            win.poll_events();
            dbg_log(&format!(
                "Mouse pos: ({}, {})",
                eng.mouse_pos().0,
                eng.mouse_pos().1
            ));
            if eng.key_pressed(Key::A) {
                self.ctr += self.swap * delta;
                if self.ctr > 1.0 {
                    self.ctr = 1.0;
                    self.swap = -self.swap;
                } else if self.ctr < 0.0 {
                    self.ctr = 0.0;
                    self.swap = -self.swap;
                }
                win.set_clear_colour(self.ctr, self.ctr, self.ctr, 1.0);
            }
            if eng.key_pressed(Key::Escape) {
                win.close();
            }
            win.clear();
            win.swap_buffers();
            0
        }
        fn destroy(&mut self, _win: &mut Window, _manager: &mut dyn EventManager, exit_code: i32) {
            dbg_log(&format!("App exiting with code {}.", exit_code));
        }
    }

    #[cfg_attr(not(target_family = "wasm"), test)]
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = start))]
    pub fn native_window_tests() {
        let app = App::new();
        let manager = DefaultEventManager::new();
        let window = Window::create(&WindowCreateArgs::new(
            "CharEngine".into(),
            400,
            400,
            WindowSizeMode::Windowed,
        ));
        window.render_loop(app, manager);
    }
}
