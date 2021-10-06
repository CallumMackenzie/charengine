use crate::data::c3d::{TriCPUBuffer, Triangle, VertexBase};
use crate::data::GPUBuffer;
use crate::platform::Window;
use gl::types::{GLuint, GLvoid};
use std::marker::PhantomData;
use std::mem::size_of;

#[allow(dead_code)]
pub struct OpenGlTriGPUBuffer<V: VertexBase> {
    vao: GLuint,
    vbo: GLuint,
    n_tris: usize,
    phantom: PhantomData<V>,
}
impl<V: VertexBase> GPUBuffer for OpenGlTriGPUBuffer<V> {
    type Data = Vec<Triangle<V>>;
    type CPUType = TriCPUBuffer<V>;
    fn new() -> Self {
        Self {
            vao: gl::NONE,
            vbo: gl::NONE,
            n_tris: 0,
            phantom: PhantomData,
        }
    }
    fn set_data(&mut self, _win: &mut Window, data: &Self::Data) {
        unsafe {
            if self.vbo == gl::NONE {
                gl::GenBuffers(1, &mut self.vbo);
            }
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            if self.n_tris == data.len() {
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
            gl::BindBuffer(gl::ARRAY_BUFFER, gl::NONE);
        }
    }
    fn get_data(&self, _win: &mut Window) -> Self::Data {
        let mut ret = Self::Data::with_capacity(self.n_tris);
        unsafe {
            ret.set_len(self.n_tris);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::GetBufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (size_of::<Triangle<V>>() * self.n_tris) as isize,
                ret.as_mut_ptr() as *mut GLvoid,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, gl::NONE);
        }
        ret
    }
}
impl<V: VertexBase> Drop for OpenGlTriGPUBuffer<V> {
    fn drop(&mut self) {
        unsafe {
            if self.vbo != gl::NONE {
                gl::DeleteBuffers(1, &mut self.vbo);
            }
            if self.vao != gl::NONE {
                gl::DeleteVertexArrays(1, &mut self.vao);
            }
        }
    }
}
