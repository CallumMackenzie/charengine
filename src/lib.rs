pub mod world;

#[cfg(any(test, target_family = "wasm"))]
mod tests {
    use charmath::linear::matrix::*;
    // use charmath::linear::vector::*;
    use charwin::cw_println;
    use charwin::data::*;
    use charwin::input::*;
    use charwin::platform::*;
    use charwin::state::*;
    use charwin::window::*;
    use std::sync::{Arc, Mutex};

    #[cfg(target_family = "wasm")]
    use wasm_bindgen::prelude::*;

    pub struct App {
        shader: Option<GPUShader>,
        tri_buffer: Option<TriGPUBuffer<VertexVT>>,
        tex: Option<Arc<Mutex<GPUTexture>>>,
        rot: f32,
    }
    impl App {
        pub fn new() -> App {
            App {
                shader: None,
                tri_buffer: None,
                tex: None,
                rot: 1.0,
            }
        }
    }
    impl State for App {
        fn initialize(&mut self, win: &mut Window, _manager: &mut dyn EventManager) -> i32 {
            win.set_clear_colour(0.2, 0.2, 0.2, 1.0);
            let vs = "#version 300 es
            precision highp float;
            layout (location = 0) in vec3 vPos;
			layout (location = 1) in vec2 vUV;
            uniform mat2 rot;
            uniform vec2 pos;
            uniform float aspect;
			out vec2 uv;
            void main() {
                vec2 trns = vPos.xy * rot * vec2(aspect, 1.0);
                gl_Position = vec4(trns.xy + pos, 0.0, 1.0);
				uv = vUV;
            }
            ";
            let fs = "#version 300 es
            precision mediump float;
            out vec4 FragColor;
			uniform sampler2D tex;
			in vec2 uv;
            void main() {
				FragColor = texture(tex, uv).rgba;
            }
            ";
            let cpu_tris = TriCPUBuffer::from_f32_array(&[
                -0.5, -0.5, 0.0, 0.0, 0.0, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5, 0.5, 0.0, 1.0, 1.0, -0.5,
                -0.5, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 1.0, 1.0, 0.5, -0.5, 0.0, 1.0, 0.0,
            ]);
            self.tri_buffer = Some(cpu_tris.to_gpu_buffer(win));
            self.tex = Some(win.load_tex_rgba_no_mip("./resource/wood.jpg"));
            self.shader = Some(GPUShader::from_sources(win, &vs, &fs));
            0
        }
        fn update(&mut self, win: &mut Window, eng: &mut dyn EventManager, delta: f64) -> i32 {
            win.poll_events();
            if eng.key_pressed(Key::Q) {
                self.rot += 3.0 * delta as f32;
            }
            if eng.key_pressed(Key::E) {
                self.rot -= 3.0 * delta as f32;
            }
            if eng.key_pressed(Key::Escape) {
                win.close();
            }
            if let (Some(shader), Some(buff)) = (self.shader.as_ref(), self.tri_buffer.as_ref()) {
                win.clear_colour();
                shader.use_shader();
                shader.set_int("tex", 0);
                shader.set_mat2f("rot", &matrices::rotation_2d(self.rot));
                shader.set_vec2f("pos", &eng.gl_mouse_vec());
                shader.set_float("aspect", eng.win_aspect_y());
                let tex = self.tex.as_ref().unwrap().lock().unwrap();
                tex.tex.bind();
                buff.vao.bind();
                shader.draw(buff.n_tris());
            }
            win.swap_buffers();
            if let (sz, true) = eng.screen_size_changed() {
                win.set_size(sz);
                win.set_resolution(sz);
            }
            0
        }
        fn destroy(&mut self, win: &mut Window, _manager: &mut dyn EventManager, exit_code: i32) {
            cw_println!("App exiting with code {}.", exit_code);
            win.clear_colour();
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
