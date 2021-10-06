#[cfg(any(test, target_family = "wasm"))]
mod tests {
    use charmath::linear::vector::*;
    use charwin::data::c3d::*;
    use charwin::data::*;
    use charwin::input::*;
    use charwin::platform::*;
    use charwin::state::*;
    use charwin::window::*;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen::prelude::*;

    #[derive(Debug)]
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
            let mut tris = Vec::new();
            tris.push(Triangle::<VertexVTN>::new());
            tris[0].v[0].v.set_x(12f32);
            tris[0].v[2].n.set_z(33.0f32);
            let cpu_tris = TriVTNCPUBuffer::from_data(&tris);
            let gpu_tris = cpu_tris.to_gpu_buffer(win);
            dbg_log(&format!("CPU: {:?}", cpu_tris.get_data()));
            dbg_log(&format!("GPU: {:?}", gpu_tris.get_data(win)));
            0
        }
        fn update(&mut self, win: &mut Window, eng: &mut dyn EventManager, delta: f64) -> i32 {
            win.poll_events();
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
    #[cfg_attr(target_family = "wasm", wasm_bindgen(start))]
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
