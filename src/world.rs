use charwin::data::{DataBuffer, GPUBuffer, TriGPUBuffer, Triangle, VertexBase, VertexVTN, TriangleBase};
use charwin::platform::Window;
use charmath::linear::vector::{Vec2f32, Vec3f32, Vec2, Vec3, Vec4, Vec4f32, Vector};
use charwin::cw_panic;
use charmath::linear::matrix::{matrices, Mat4, SquareMatrix, Matrix};
use charwin::window::EventManager;
use charwin::input::Key;


pub trait MeshBase<V: VertexBase>: Sized {
    fn new(win: &mut Window) -> Self;
    fn n_tris(&self) -> i32;
    fn set_data(&mut self, data: &Vec<Triangle<V>>);

    fn from_data(win: &mut Window, data: &Vec<Triangle<V>>) -> Self {
        let mut ret = Self::new(win);
        ret.set_data(data);
        ret
    }
    fn tris_from_obj_data(data: &str) -> Vec<Triangle<VertexVTN>> {
		let mut verts = Vec::new();
		let mut normals = Vec::new();
		let mut texs = Vec::new();
        let mut tris = Vec::new();

		let lines: Vec<&str> = data.lines().collect();
		let has_normals = data.contains(&"vn");
		let has_uv = data.contains(&"vt");
		for line in lines {
			let line_bytes = line.as_bytes();
			match line_bytes[0] as char {
				'v' => {
					let seg: Vec<&str> = line.split_whitespace().collect();
					match line_bytes[1] as char {
					't' => {
						texs.push(
							Vec2f32::new(
								seg[1].parse::<f32>().unwrap(),
								seg[2].parse::<f32>().unwrap(),
							)
						);
					},
					'n' => {
						normals.push(
							Vec3f32::new(
								seg[1].parse::<f32>().unwrap(),
								seg[2].parse::<f32>().unwrap(),
								seg[3].parse::<f32>().unwrap(),
							)
						)
					},
					_ => {
						verts.push(
							Vec3f32::new(
								seg[1].parse::<f32>().unwrap(),
								seg[2].parse::<f32>().unwrap(),
								seg[3].parse::<f32>().unwrap(),
							)
						)
					}
				}
			},
				'f' => {
					let params = 1 + has_normals as usize + has_uv as usize;
					let mut vals = Vec::with_capacity(9);
					let seg: Vec<String> = line.replace("f", "").as_str().split(|ch| ch == ' ' || ch == '/').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

					for i in 0..seg.len() {
						vals.push(seg[i].as_str().parse::<usize>().unwrap_or_else(|_| {
							cw_panic!("Could not parse index: \"{}\". Line: \"{}\"", seg[i], line);
						}));
					}

					let mut push = Triangle::<VertexVTN>::new();
					for k in 0..3 {
                    	push.v[k].v = verts[vals[params * k] - 1];
						if has_uv {
							push.v[k].t = texs[vals[(params * k) + 1] - 1];
						} else if has_normals {
							push.v[k].n = normals[vals[(params * k) + 1] - 1];
						}
						if has_normals && has_uv {
							push.v[k].n = normals[vals[(params * k) + 2] - 1];
						}
					}
					tris.push(push);
				},
				_ => {}
			}
		}
		tris
    }
}

pub struct Mesh3D<V: VertexBase> {
    pub buffer: TriGPUBuffer<V>,
}
impl<V: VertexBase> MeshBase<V> for Mesh3D<V> {
    fn new(win: &mut Window) -> Self {
        Self {
            buffer: TriGPUBuffer::<V>::new(win),
        }
    }
    fn n_tris(&self) -> i32 {
        self.buffer.n_tris()
    }
    fn set_data(&mut self, data: &Vec<Triangle<V>>) {
        self.buffer.set_data(data);
    }
}

pub struct PerspectiveCamera3D {
	pub fov: f32,
	pub near: f32,
	pub far: f32,
	pub pos: Vec3f32,
	pub rot: Vec3f32,
}
impl PerspectiveCamera3D {
	pub fn projection(&self, aspect: f32) -> Mat4<f32> {
		matrices::perspective(self.fov, aspect, self.near, self.far)
	}
	pub fn camera_matrix(&self) -> Mat4<f32> {
		let up = Vec3f32::new(0.0, 1.0, 0.0);
        let target = Vec4f32::new(0.0, 0.0, 1.0, 1.0);
		let camera_rot = matrices::rotation_euler(&self.rot);
        let camera_rot_vec = camera_rot.mul_col_vec(&target);
        let target = &self.pos + Vec3f32::new_vec(&camera_rot_vec);
        matrices::look_at_3d(&self.pos, &target, &up)
	}
	pub fn view(&self) -> Mat4<f32> {
		self.camera_matrix().inverse()
	}
	pub fn look_vector(&self) -> Vec3f32 {
		let target = Vec4f32::new(0.0, 0.0, 1.0, 1.0);
        let m_rot = matrices::rotation_euler(&self.rot);
        Vec3f32::new_vec(&m_rot.mul_col_vec(&target))
	}
	pub fn debug_controls(&mut self, man: &dyn EventManager, delta: f32, move_speed: f32, rot_speed: f32) {
		let clv = self.look_vector();
        let mut forward = Vec3f32::new(0.0, 0.0, 0.0);
        let up = Vec3f32::new(0.0, 1.0, 0.0);
        let mut rotate = Vec3f32::new(0.0, 0.0, 0.0);
        if man.key_pressed(Key::W) {
            forward += &clv;
		}
        if man.key_pressed(Key::S) {
            forward -= &clv;
		}
        if man.key_pressed(Key::D) {
            forward -= clv.cross(&up);
		}
        if man.key_pressed(Key::A) {
            forward += clv.cross(&up);
		}
        if man.key_pressed(Key::Q) || man.key_pressed(Key::Space) {
            forward += Vec3f32::new(0.0, 1.0, 0.0);
		}
        if man.key_pressed(Key::E) {
            forward += Vec3f32::new(0.0, -1.0, 0.0);
		}
        if man.key_pressed(Key::Left) {
            rotate.set_y(-rot_speed);
		}
        if man.key_pressed(Key::Right) {
            rotate.set_y(rot_speed);
		}
        if man.key_pressed(Key::Up) {
            rotate.set_x(rot_speed);
		}
        if man.key_pressed(Key::Down) {
            rotate.set_x(-rot_speed);
		}
        self.rot += rotate * delta;
        self.pos += forward.normalized() * move_speed * delta;
		const NEAR_PI_OVER_2: f32 = std::f32::consts::PI * 0.48;
        if self.rot.get_x() >= NEAR_PI_OVER_2 {
			self.rot.set_x(NEAR_PI_OVER_2);
		}
        if self.rot.get_x() <= -NEAR_PI_OVER_2 {
			self.rot.set_x(-NEAR_PI_OVER_2);
		}
        if f32::abs(self.rot.get_y()) >= std::f32::consts::PI * 2.0 {
			self.rot.set_y(0.0);
		}
        if f32::abs(self.rot.get_z()) >= std::f32::consts::PI * 2.0 {
			self.rot.set_z(0.0);
		}
	}
}