#[cfg(any(test, target_family = "wasm"))]
mod tests {
    use charwin::platform::Window;
    use charwin::state::State;
    use charwin::window::{
        AbstractWindow, AbstractWindowFactory, WindowCreateArgs, WindowSizeMode,
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
        fn initialize(&mut self, _win: &mut Window) -> i32 {
            0
        }
        fn update(&mut self, win: &mut Window, delta: f64) -> i32 {
            if win.should_close() {
                1
            } else {
                win.clear();
                win.swap_buffers();
                win.poll_events();
                self.ctr += self.swap * delta;
                if self.ctr > 1.0 {
                    self.ctr = 1.0;
                    self.swap = -self.swap;
                } else if self.ctr < 0.0 {
                    self.ctr = 0.0;
                    self.swap = -self.swap;
                }
                win.set_clear_colour(self.ctr, self.ctr, self.ctr, 1.0);
                0
            }
        }
        fn destroy(&mut self, _win: &mut Window, exit_code: i32) {
            println!("App exiting with code {}.", exit_code);
        }
    }

    #[cfg_attr(not(target_family = "wasm"), test)]
    #[cfg_attr(target_family = "wasm", wasm_bindgen(start))]
    pub fn native_window_tests() {
        let app: App = App::new();
        let window = Window::create(&WindowCreateArgs::new(
            "CharEngine".into(),
            400,
            400,
            WindowSizeMode::Windowed,
        ));
        window.render_loop(app);
    }
}
