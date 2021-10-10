use crate::platform::Window;
use charmath::linear::matrix::{Mat2F, Mat4F};
use charmath::linear::vector::{Vec2f32, Vec2i32, Vec3f32, Vec3i32, Vec4f32, Vec4i32};

pub trait GPUShaderBase: Sized {
    fn new(win: &Window) -> Self;
    fn compile(&mut self, vertex: &str, fragment: &str);
    fn use_shader(&self);
    fn set_4f32(&self, name: &str, v: &Vec4f32);
    fn set_3f32(&self, name: &str, v: &Vec3f32);
    fn set_2f32(&self, name: &str, v: &Vec2f32);
    fn set_1f32(&self, name: &str, v: f32);
    fn set_4i32(&self, name: &str, v: &Vec4i32);
    fn set_3i32(&self, name: &str, v: &Vec3i32);
    fn set_2i32(&self, name: &str, v: &Vec2i32);
    fn set_1i32(&self, name: &str, v: i32);
    fn set_mat4f32(&self, name: &str, v: &Mat4F);
    fn set_mat2f32(&self, name: &str, v: &Mat2F);
    fn draw_from(&self, start: u32, n_tris: u32);

    fn draw(&self, n_tris: u32) {
        self.draw_from(0, n_tris);
    }
    fn from_sources(win: &Window, vertex: &str, fragment: &str) -> Self {
        let mut ret = Self::new(win);
        ret.compile(vertex, fragment);
        ret
    }
    fn set_bool(&self, name: &str, v: bool) {
        self.set_1i32(name, v as i32);
    }
}
