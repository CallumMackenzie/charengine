pub mod world;

#[cfg(any(test, target_family = "wasm"))]
mod tests {
    use crate::world::*;
    use charmath::linear::matrix::*;
    use charmath::linear::vector::*;
	use charmath::linear::quaternion::Quaternionf32;
    use charwin::cw_println;
    use charwin::data::*;
    use charwin::input::*;
    use charwin::platform::*;
    use charwin::state::*;
    use charwin::window::*;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen::prelude::*;

    pub struct AppData {
        shader: GPUShader,
        mshs: Vec<Object3D>,
        camera: PerspectiveCamera3D,
		ctr: f32
    }
    pub struct App {
        data: Option<AppData>,
    }
    impl State for App {
        fn initialize(&mut self, win: &mut Window, _manager: &mut dyn EventManager) -> i32 {
            win.set_clear_colour(0.2, 0.2, 0.2, 1.0);
            let vs = "#version 300 es
            precision highp float;

			struct Camera {
				mat4 projection;
				mat4 view;
				vec3 position;
			};
			struct Mesh {
				mat4 transform;
				mat4 rotation;
			};

            layout (location = 0) in vec3 vPos;
			layout (location = 1) in vec2 vUV;
			layout (location = 2) in vec3 vNorm;

			uniform Camera camera;
			uniform Mesh mesh;

			out vec2 uv;
			out vec3 norm;
            void main() {
				gl_Position = camera.projection * camera.view * mesh.transform * vec4(vPos, 1.0);

				uv = vUV;
				norm = (mesh.rotation * vec4(vNorm, 1.0)).xyz;
            }
            ";
            let fs = "#version 300 es
            precision mediump float;
            out vec4 FragColor;
			
			struct Material {
				sampler2D diffuse;
			};

			uniform vec3 cameraPos;
			uniform Material material;
			in vec3 norm;
			in vec2 uv;

			vec3 getDiffuse(vec4 tex, vec3 lightDir, vec3 normal) {
				float diffuseCoeff = max(dot(normal, lightDir), 0.0);
				return tex.xyz * diffuseCoeff;
			}

			vec3 getAmbient(vec4 tex) {
				return vec3(0.2, 0.2, 0.2);
			}

			vec3 getSpecular(vec4 tex, vec3 lightDir, vec3 normal) {
				// TODO
				return tex.xyz;
			}

            void main() {
				vec3 lightDir = vec3(0.0, 1.0, 0.0);
				vec4 diffuseTexture = texture(material.diffuse, uv).rgba;

				vec3 ambient = getAmbient(diffuseTexture);
				vec3 diffuse = getDiffuse(diffuseTexture, lightDir, norm);
				vec3 specular = getSpecular(diffuseTexture, lightDir, norm);
				FragColor = vec4(ambient + diffuse, 1.0);
            }
            ";
            self.data = Some(AppData {
                shader: GPUShader::from_sources(win, &vs, &fs),
                mshs: vec![
                    Object3D::new(
						win,
                        std::include_str!("../resource/torusnt.obj"),
                        "./resource/wood.jpg",
                    ),
                    // Object3D::new(
					// 	win,
                    //     std::include_str!("../resource/cubent.obj"),
                    //     "./resource/dirt.jpg",
                    // ),
                ],
                camera: PerspectiveCamera3D {
                    fov: 75.0,
                    near: 0.1,
                    far: 100.0,
                    pos: Vec3f32::new(0.0, 0.0, 0.0),
                    rot: Vec3f32::new(0.0, 0.0, 0.0),
                },
				ctr: 0.0
            });
            let mut context = win.get_gl_context();
            context.enable(GlFeature::DepthTest);
            context.default_depth_func();
            0
        }
        fn update(&mut self, win: &mut Window, eng: &mut dyn EventManager, delta: f64) -> i32 {
            if eng.key_pressed(Key::Escape) {
                win.close();
            }
            if let Some(data) = self.data.as_mut() {
				data.ctr += (5.0 * delta) as f32;
				data.mshs[0].rot = Quaternionf32::angle_axis(data.ctr, &Vec3f32::new(1.0, 1.0, 0.0));

                data.camera.debug_controls(eng, delta as f32, 4.0, 1.8);
                win.clear(&[GlClearMask::Color, GlClearMask::Depth]);
                data.shader.use_shader();
                data.shader.set_int("tex", 0);
                data.shader.set_mat4f("camera.view", &data.camera.view());
                data.shader.set_mat4f(
                    "camera.projection",
                    &data.camera.projection(eng.win_aspect_y()),
                );
                for mesh in &data.mshs {
                    mesh.render(&data.shader);
                }
                win.swap_buffers();
                if let (sz, true) = eng.screen_size_changed() {
                    win.set_size(sz);
                    win.set_resolution(sz);
                }
            }
            0
        }
        fn destroy(&mut self, win: &mut Window, _man: &mut dyn EventManager, exit_code: i32) {
            cw_println!("App exiting with code {}.", exit_code);
            win.clear_colour();
        }
    }

    #[cfg_attr(not(target_family = "wasm"), test)]
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = start))]
    pub fn native_window_tests() {
        let app = App { data: None };
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
