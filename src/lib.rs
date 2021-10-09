#[cfg(any(test, target_family = "wasm"))]
mod tests {
    use charmath::linear::matrix::*;
    // use charmath::linear::vector::*;
    use charwin::data::c3d::*;
    use charwin::data::*;
    use charwin::input::*;
    use charwin::platform::*;
    use charwin::state::*;
    use charwin::window::*;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen::prelude::*;

    pub struct App {
        shader: Option<GPUShader>,
        tri_buffer: Option<TriGPUBuffer<VertexV>>,
        rot: f32,
    }
    impl App {
        pub fn new() -> App {
            App {
                shader: None,
                tri_buffer: None,
                rot: 1.0,
            }
        }
    }
    impl State for App {
        fn initialize(&mut self, win: &mut Window, _manager: &mut dyn EventManager) -> i32 {
            win.set_clear_colour(0.1, 0.1, 0.5, 1.0);
            let vs = "#version 300 es
            precision highp float;
            layout (location = 0) in vec3 aPos;
            uniform mat2 rot;
            uniform vec2 pos;
            void main() {
                gl_Position = vec4(aPos.xy * rot + pos, 0.0, 1.0);
            }
            ";
            let fs = "#version 300 es
            precision mediump float;
            out vec4 FragColor;
            void main() {
               FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
            }
            ";
            let cpu_tris = TriCPUBuffer::<VertexV>::from_f32_array(&[
                -0.5, -0.5, 0.0, -0.5, 0.5, 0.0, 0.5, 0.5, 0.0, -0.5, -0.5, 0.0, 0.5, 0.5, 0.0,
                0.5, -0.5, 0.0,
            ]);
            self.tri_buffer = Some(cpu_tris.to_gpu_buffer(win));
            if let Some(buff) = self.tri_buffer.as_ref() {
                buff.set_std_attib_ptrs();
            }
            self.shader = Some(GPUShader::from_sources(win, &vs, &fs));
            0
        }
        fn update(&mut self, win: &mut Window, eng: &mut dyn EventManager, delta: f64) -> i32 {
            win.poll_events();
            if eng.key_pressed(Key::Q) {
                self.rot += 1.0 * delta as f32;
            }
            if eng.key_pressed(Key::E) {
                self.rot -= 1.0 * delta as f32;
            }
            if let (Some(shader), Some(buff)) = (self.shader.as_ref(), self.tri_buffer.as_ref()) {
                win.clear();
                shader.use_shader();
                shader.set_mat2f32("rot", &matrices::rotation_2d(self.rot));
                shader.set_2f32("pos", &eng.gl_mouse_vec());
                buff.bind_vao();
                shader.draw(buff.n_tris());
            }
            win.swap_buffers();
            if let (sz, true) = eng.screen_size_changed() {
                win.set_size(sz);
                win.set_resolution(sz);
            }
            if eng.key_pressed(Key::Escape) {
                win.close();
            }
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
