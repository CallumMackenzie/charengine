use crate::data::c3d::{TriCPUBuffer, Triangle, VertexBase};
use crate::data::{GPUBuffer, GPUShaderBase};
use crate::platform::Window;
use charmath::linear::matrix::{Mat2F, Mat4F, MatrixBase};
use charmath::linear::vector::{
    Vec2, Vec2f32, Vec2i32, Vec3, Vec3f32, Vec3i32, Vec4, Vec4f32, Vec4i32,
};
use gl::types::{GLchar, GLint, GLsizei, GLuint, GLvoid};
use std::ffi::CString;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr;
use std::str;

#[allow(dead_code)]
pub struct OpenGlTriGPUBuffer<V: VertexBase> {
    vao: GLuint,
    vbo: GLuint,
    n_tris: usize,
    phantom: PhantomData<V>,
}
impl<V: VertexBase> OpenGlTriGPUBuffer<V> {
    pub fn bind_vbo(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        }
    }
    pub fn bind_vao(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }
    pub fn unbind_vao(&self) {
        unsafe {
            gl::BindVertexArray(gl::NONE);
        }
    }
    pub fn unbind_vbo(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, gl::NONE);
        }
    }
    pub fn set_attrib_ptr(&self, index: usize, size: usize, step: usize, offset: usize) {
        self.bind_vao();
        self.bind_vbo();
        unsafe {
            gl::VertexAttribPointer(
                index as GLuint,
                size as GLint,
                gl::FLOAT,
                gl::FALSE,
                step as GLsizei,
                (offset as GLuint) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(index as GLuint);
        }
        self.unbind_vbo();
        self.unbind_vao();
    }
    pub fn remove_attrib_ptr(&self, index: u32) {
        self.bind_vao();
        self.bind_vbo();
        unsafe {
            if self.vao != gl::NONE {
                gl::DisableVertexAttribArray(index);
            }
        }
        self.unbind_vbo();
        self.unbind_vao();
    }
    pub fn set_std_attib_ptrs(&self) {
        self.set_attrib_ptr(0, 3, size_of::<V>(), 0);
    }
    pub fn n_tris(&self) -> i32 {
        self.n_tris as i32
    }
}
impl<V: VertexBase> GPUBuffer for OpenGlTriGPUBuffer<V> {
    type Data = Vec<Triangle<V>>;
    type CPUType = TriCPUBuffer<V>;
    fn new(_: &mut Window) -> Self {
        let mut ret = Self {
            vao: gl::NONE,
            vbo: gl::NONE,
            n_tris: 0,
            phantom: PhantomData,
        };
        unsafe {
            gl::GenVertexArrays(1, &mut ret.vao);
            gl::GenBuffers(1, &mut ret.vbo);
        }
        ret
    }
    fn set_data(&mut self, data: &Self::Data) {
        self.bind_vao();
        self.bind_vbo();
        unsafe {
            if self.n_tris == data.len() && data.len() != 0 {
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (size_of::<Triangle<V>>() * data.len()) as isize,
                    data.as_ptr() as *const GLvoid,
                );
            } else {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (size_of::<Triangle<V>>() * data.len()) as isize,
                    data.as_ptr() as *const GLvoid,
                    gl::STATIC_DRAW,
                );
            }
            self.n_tris = data.len();
        }
        self.unbind_vbo();
        self.unbind_vao();
    }
    fn get_data(&self) -> Self::Data {
        self.bind_vbo();
        let mut ret = Self::Data::with_capacity(self.n_tris);
        unsafe {
            ret.set_len(self.n_tris);
            gl::GetBufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (size_of::<Triangle<V>>() * self.n_tris) as isize,
                ret.as_mut_ptr() as *mut GLvoid,
            );
        }
        self.unbind_vbo();
        ret
    }
}
impl<V: VertexBase> Drop for OpenGlTriGPUBuffer<V> {
    fn drop(&mut self) {
        unsafe {
            if self.vao != gl::NONE {
                gl::DeleteVertexArrays(1, &mut self.vao);
            }
            if self.vbo != gl::NONE {
                gl::DeleteBuffers(1, &mut self.vbo);
            }
        }
    }
}
pub struct OpenGlGPUShader {
    program: GLuint,
}
impl OpenGlGPUShader {
    fn gsl(&self, name: &str) -> GLint {
        let c_str = CString::new(name.as_bytes()).unwrap();
        unsafe { gl::GetUniformLocation(self.program, c_str.as_ptr() as *const GLchar) }
    }
    fn compile_shader(src: &str, shader_type: GLuint) -> GLuint {
        unsafe {
            let shader = gl::CreateShader(shader_type);
            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "{}",
                    str::from_utf8(buf.as_slice())
                        .ok()
                        .expect("ShaderInfoLog not valid utf8")
                );
            }
            shader
        }
    }
}
impl GPUShaderBase for OpenGlGPUShader {
    fn new(_: &Window) -> Self {
        Self { program: gl::NONE }
    }
    fn compile(&mut self, vsrc: &str, fsrc: &str) {
        if self.program != gl::NONE {
            unsafe {
                gl::DeleteProgram(self.program);
            }
            self.program = gl::NONE;
        }
        unsafe {
            self.program = gl::CreateProgram();
            let vs = Self::compile_shader(vsrc, gl::VERTEX_SHADER);
            if vs == gl::NONE {
                panic!("Verex shader could not be created.");
            }
            let fs = Self::compile_shader(fsrc, gl::FRAGMENT_SHADER);
            if fs == gl::NONE {
                panic!("Fragment shader could not be created.");
            }
            gl::AttachShader(self.program, vs);
            gl::AttachShader(self.program, fs);
            gl::LinkProgram(self.program);
            let mut link_success: GLint = gl::TRUE as GLint;
            gl::GetProgramiv(self.program, gl::LINK_STATUS, &mut link_success);
            if link_success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetProgramiv(self.program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);
                gl::GetProgramInfoLog(
                    self.program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "{}",
                    str::from_utf8(buf.as_slice())
                        .ok()
                        .expect("ProgramInfoLog not valid utf8")
                );
            }
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
        }
    }
    fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }
    fn draw(&self, n_tris: i32) {
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, n_tris * 3);
        }
    }
    fn set_4f32(&self, name: &str, v: &Vec4f32) {
        unsafe {
            gl::Uniform4f(self.gsl(name), v.get_x(), v.get_y(), v.get_z(), v.get_w());
        }
    }
    fn set_3f32(&self, name: &str, v: &Vec3f32) {
        unsafe {
            gl::Uniform3f(self.gsl(name), v.get_x(), v.get_y(), v.get_z());
        }
    }
    fn set_2f32(&self, name: &str, v: &Vec2f32) {
        unsafe {
            gl::Uniform2f(self.gsl(name), v.get_x(), v.get_y());
        }
    }
    fn set_1f32(&self, name: &str, v: f32) {
        unsafe {
            gl::Uniform1f(self.gsl(name), v);
        }
    }
    fn set_4i32(&self, name: &str, v: &Vec4i32) {
        unsafe {
            gl::Uniform4i(self.gsl(name), v.get_x(), v.get_y(), v.get_z(), v.get_w());
        }
    }
    fn set_3i32(&self, name: &str, v: &Vec3i32) {
        unsafe {
            gl::Uniform3i(self.gsl(name), v.get_x(), v.get_y(), v.get_z());
        }
    }
    fn set_2i32(&self, name: &str, v: &Vec2i32) {
        unsafe {
            gl::Uniform2i(self.gsl(name), v.get_x(), v.get_y());
        }
    }
    fn set_1i32(&self, name: &str, v: i32) {
        unsafe {
            gl::Uniform1i(self.gsl(name), v);
        }
    }
    fn set_mat4f32(&self, name: &str, v: &Mat4F) {
        unsafe {
            gl::UniformMatrix4fv(self.gsl(name), 1, 0, v.flatten().as_ptr());
        }
    }
    fn set_mat2f32(&self, name: &str, v: &Mat2F) {
        unsafe {
            gl::UniformMatrix2fv(self.gsl(name), 1, 0, v.flatten().as_ptr());
        }
    }
}
impl Drop for OpenGlGPUShader {
    fn drop(&mut self) {
        unsafe {
            if self.program != gl::NONE {
                gl::DeleteProgram(self.program);
            }
        }
    }
}
