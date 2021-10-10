use crate::char_panic;
use crate::platform::{Buffer, Program, Shader, VertexArray, Window};
use crate::window::{
    GlBindable, GlBuffer, GlBufferType, GlDrawMode, GlProgram, GlShader, GlShaderType,
    GlStorageMode, GlVertexArray,
};
use charmath::linear::matrix::{Mat2F, Mat4F, MatrixBase};
use charmath::linear::vector::{
    Vec2, Vec2f32, Vec2i32, Vec3, Vec3f32, Vec3i32, Vec4, Vec4f32, Vec4i32,
};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Index, IndexMut};

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

pub trait CPUBuffer: Sized {
    type Data: Sized;
    type GPUType: GPUBuffer<Data = Self::Data>;
    fn new() -> Self;
    fn set_data(&mut self, data: &Self::Data);
    fn get_data(&self) -> Self::Data;

    fn from_data(data: &Self::Data) -> Self {
        let mut ret = Self::new();
        ret.set_data(data);
        ret
    }
    fn to_gpu_buffer(&self, win: &mut Window) -> Self::GPUType {
        Self::GPUType::from_data(win, &self.get_data())
    }
}
pub trait GPUBuffer: Sized {
    type Data: Sized;
    type CPUType: CPUBuffer<Data = Self::Data>;
    fn new(win: &mut Window) -> Self;
    fn set_data(&mut self, data: &Self::Data);
    fn get_data(&self) -> Self::Data;

    fn from_data(win: &mut Window, data: &Self::Data) -> Self {
        let mut ret = Self::new(win);
        ret.set_data(data);
        ret
    }
    fn to_cpu_buffer(&self) -> Self::CPUType {
        Self::CPUType::from_data(&self.get_data())
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct VertexAttrib(pub u32, pub u32, pub usize, pub usize);
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl VertexAttrib {
    #[wasm_bindgen(constructor)]
    pub fn wnew(index: i32, size: i32, step: i32, offset: i32) -> Self {
        VertexAttrib(index as u32, size as u32, step as usize, offset as usize)
    }
}

pub trait VertexBase: Copy + Sized {
    fn new() -> Self;
    fn float_size() -> usize;
    fn get_attribs() -> Vec<VertexAttrib>;
    fn to_f32_array(&self) -> Vec<f32>;
    fn from_f32_array(arr: &[f32]) -> Self;
}

pub trait TriangleBase: Sized {
    type Vert: VertexBase;
    fn new() -> Self;
    fn get_vertecies(&self) -> [Self::Vert; 3];
    fn set_vertecies(&mut self, v: &[Self::Vert; 3]);

    fn from_verts(verts: &[Self::Vert; 3]) -> Self {
        let mut ret = Self::new();
        ret.set_vertecies(verts);
        ret
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Triangle<V: VertexBase> {
    pub v: [V; 3],
}
impl<V: VertexBase> TriangleBase for Triangle<V> {
    type Vert = V;
    fn new() -> Self {
        Self {
            v: [Self::Vert::new(); 3],
        }
    }
    fn get_vertecies(&self) -> [V; 3] {
        [self.v[0], self.v[1], self.v[2]]
    }
    fn set_vertecies(&mut self, nv: &[V; 3]) {
        self.v = *nv;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TriCPUBuffer<V: VertexBase> {
    tris: Vec<Triangle<V>>,
}
impl<V: VertexBase> TriCPUBuffer<V> {
    pub fn n_tris(&self) -> usize {
        self.tris.len()
    }
    pub fn to_f32_array(&self) -> Vec<f32> {
        let mut ret = Vec::with_capacity(self.tris.len() * 3 * V::float_size());
        for i in 0..self.tris.len() {
            for j in 0..3 {
                for float in self.tris[i].v[j].to_f32_array() {
                    ret.push(float);
                }
            }
        }
        ret
    }
    pub fn from_f32_array(arr: &[f32]) -> Self {
        let mut data = Vec::new();
        for i in 0..(arr.len() / V::float_size() / 3) {
            let mut tri = Triangle::<V>::new();
            let arr_index = i * V::float_size() * 3;
            for j in 0..3 {
                let start_slice = arr_index + (V::float_size() * j);
                let end_slice = start_slice + V::float_size();
                tri.v[j] = V::from_f32_array(&arr[start_slice..end_slice]);
            }
            data.push(tri);
        }
        Self { tris: data }
    }
    pub fn data_ptr(&self) -> *const f32 {
        self.tris.as_ptr() as *const f32
    }
}
impl<V: VertexBase> CPUBuffer for TriCPUBuffer<V> {
    type Data = Vec<Triangle<V>>;
    type GPUType = TriGPUBuffer<V>;
    fn new() -> Self {
        TriCPUBuffer { tris: Vec::new() }
    }
    fn set_data(&mut self, data: &Self::Data) {
        self.tris.clear();
        for i in 0..data.len() {
            self.tris.push(data[i]);
        }
    }
    fn get_data(&self) -> Self::Data {
        let mut ret = Vec::with_capacity(self.tris.len());
        for i in 0..self.tris.len() {
            ret.push(self.tris[i]);
        }
        ret
    }
    fn to_gpu_buffer(&self, win: &mut Window) -> Self::GPUType {
        Self::GPUType::from_data(win, &self.tris)
    }
}
impl<V: VertexBase> Index<usize> for TriCPUBuffer<V> {
    type Output = Triangle<V>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.tris[index]
    }
}
impl<V: VertexBase> IndexMut<usize> for TriCPUBuffer<V> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.tris[index]
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VertexV2D {
    pub v: Vec2f32,
}
impl VertexBase for VertexV2D {
    fn new() -> Self {
        Self {
            v: Vec2f32::new(0f32, 0f32),
        }
    }
    fn get_attribs() -> Vec<VertexAttrib> {
        vec![VertexAttrib(0, 2, size_of::<Self>(), 0)]
    }
    fn float_size() -> usize {
        2usize
    }
    fn to_f32_array(&self) -> Vec<f32> {
        vec![self.v[0], self.v[1]]
    }
    fn from_f32_array(arr: &[f32]) -> Self {
        let mut ret = Self::new();
        for i in 0..2 {
            ret.v[i] = arr[i];
        }
        ret
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VertexV {
    pub v: Vec3f32,
}
impl VertexBase for VertexV {
    fn new() -> Self {
        Self {
            v: Vec3f32::new(0f32, 0f32, 0f32),
        }
    }
    fn get_attribs() -> Vec<VertexAttrib> {
        vec![VertexAttrib(0, 3, size_of::<Self>(), 0)]
    }
    fn float_size() -> usize {
        3usize
    }
    fn to_f32_array(&self) -> Vec<f32> {
        vec![self.v[0], self.v[1], self.v[2]]
    }
    fn from_f32_array(arr: &[f32]) -> Self {
        let mut ret = Self::new();
        for i in 0..3 {
            ret.v[i] = arr[i];
        }
        ret
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VertexVT {
    pub v: Vec3f32,
    pub t: Vec2f32,
}
impl VertexBase for VertexVT {
    fn new() -> Self {
        Self {
            v: Vec3f32::new(0f32, 0f32, 0f32),
            t: Vec2f32::new(0f32, 0f32),
        }
    }
    fn get_attribs() -> Vec<VertexAttrib> {
        let step = size_of::<Self>();
        vec![
            VertexAttrib(0, 3, step, 0),
            VertexAttrib(1, 2, step, size_of::<Vec3f32>()),
        ]
    }
    fn float_size() -> usize {
        5usize
    }
    fn to_f32_array(&self) -> Vec<f32> {
        vec![self.v[0], self.v[1], self.v[2], self.t[0], self.t[1]]
    }
    fn from_f32_array(arr: &[f32]) -> Self {
        let mut ret = Self::new();
        for i in 0..3 {
            ret.v[i] = arr[i];
        }
        for i in 0..2 {
            ret.t[i] = arr[3 + i];
        }
        ret
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VertexVTN {
    pub v: Vec3f32,
    pub t: Vec2f32,
    pub n: Vec3f32,
}
impl VertexBase for VertexVTN {
    fn new() -> Self {
        Self {
            v: Vec3f32::new(0f32, 0f32, 0f32),
            t: Vec2f32::new(0f32, 0f32),
            n: Vec3f32::new(0f32, 0f32, 0f32),
        }
    }
    fn get_attribs() -> Vec<VertexAttrib> {
        let step = size_of::<Self>();
        vec![
            VertexAttrib(0, 3, step, 0),
            VertexAttrib(1, 2, step, size_of::<Vec3f32>()),
            VertexAttrib(2, 3, step, size_of::<Vec3f32>() + size_of::<Vec2f32>()),
        ]
    }
    fn float_size() -> usize {
        8usize
    }
    fn to_f32_array(&self) -> Vec<f32> {
        vec![
            self.v[0], self.v[1], self.v[2], self.t[0], self.t[1], self.n[0], self.n[1], self.n[2],
        ]
    }
    fn from_f32_array(arr: &[f32]) -> Self {
        let mut ret = Self::new();
        for i in 0..3 {
            ret.v[i] = arr[i];
        }
        for i in 0..2 {
            ret.t[i] = arr[3 + i];
        }
        for i in 0..3 {
            ret.n[i] = arr[5 + i];
        }
        ret
    }
}

pub struct GPUShader {
    pub prog: Program,
}
impl GPUShader {
    pub fn new(w: &Window) -> Self {
        Self {
            prog: Program::new(w),
        }
    }
    pub fn compile(&self, w: &Window, v: &str, f: &str) {
        match (
            Shader::from_source(w, GlShaderType::Vertex, v),
            Shader::from_source(w, GlShaderType::Fragment, f),
        ) {
            (Ok(vs), Ok(fs)) => {
                self.prog.attach_shader(&vs);
                self.prog.attach_shader(&fs);
                self.prog.link_program();
                if let Some(err) = self.prog.get_link_status() {
                    char_panic!("Error linking program: Program({})", err);
                }
            }
            (Err(vs), Err(fs)) => {
                char_panic!(
                    "Error compiling vertex and fragment shaders: Verex({}), Fragment({})",
                    vs,
                    fs
                );
            }
            (Err(vs), Ok(_)) => {
                char_panic!("Error compiling vertex shader: Vertex({})", vs);
            }
            (Ok(_), Err(fs)) => {
                char_panic!("Error compiling framgnet shader: Fragment({})", fs);
            }
        }
    }
    pub fn use_shader(&self) {
        self.prog.bind();
    }
    pub fn draw(&self, nt: i32) {
        self.draw_from(0, nt);
    }
    pub fn draw_from(&self, start: i32, n_tris: i32) {
        self.prog
            .draw_arrays(GlDrawMode::Triangles, start, n_tris * 3);
    }
    pub fn from_sources(w: &Window, v: &str, f: &str) -> Self {
        let ret = Self::new(w);
        ret.compile(w, v, f);
        ret
    }
    pub fn set_vec4f(&self, name: &str, vec: &Vec4f32) {
        self.prog
            .uniform_4f(&self.prog.shader_loc(name), vec.as_tuple());
    }
    pub fn set_vec3f(&self, name: &str, vec: &Vec3f32) {
        self.prog
            .uniform_3f(&self.prog.shader_loc(name), vec.as_tuple());
    }
    pub fn set_vec2f(&self, name: &str, vec: &Vec2f32) {
        self.prog
            .uniform_2f(&self.prog.shader_loc(name), vec.as_tuple());
    }
    pub fn set_float(&self, name: &str, vec: f32) {
        self.prog.uniform_1f(&self.prog.shader_loc(name), vec);
    }
    pub fn set_vec4i(&self, name: &str, vec: &Vec4i32) {
        self.prog
            .uniform_4i(&self.prog.shader_loc(name), vec.as_tuple());
    }
    pub fn set_vec3i(&self, name: &str, vec: &Vec3i32) {
        self.prog
            .uniform_3i(&self.prog.shader_loc(name), vec.as_tuple());
    }
    pub fn set_vec2i(&self, name: &str, vec: &Vec2i32) {
        self.prog
            .uniform_2i(&self.prog.shader_loc(name), vec.as_tuple());
    }
    pub fn set_int(&self, name: &str, vec: i32) {
        self.prog.uniform_1i(&self.prog.shader_loc(name), vec);
    }
    pub fn set_mat4f(&self, name: &str, mat: &Mat4F) {
        self.prog
            .uniform_mat4f(&self.prog.shader_loc(name), &mat.flatten());
    }
    pub fn set_mat2f(&self, name: &str, mat: &Mat2F) {
        self.prog
            .uniform_mat2f(&self.prog.shader_loc(name), &mat.flatten());
    }
}

pub struct TriGPUBuffer<V: VertexBase> {
    pub vbo: Buffer,
    pub vao: VertexArray,
    n_tris: i32,
    phantom: PhantomData<V>,
}
impl<V: VertexBase> GPUBuffer for TriGPUBuffer<V> {
    type Data = Vec<Triangle<V>>;
    type CPUType = TriCPUBuffer<V>;
    fn new(win: &mut Window) -> Self {
        Self {
            vbo: Buffer::new(win, GlBufferType::ArrayBuffer),
            vao: VertexArray::new(win),
            n_tris: 0,
            phantom: PhantomData,
        }
    }
    fn set_data(&mut self, data: &Self::Data) {
        self.n_tris = data.len() as i32;
        self.vao.bind();
        self.vbo.bind();
        for attrib in V::get_attribs() {
            self.vao.attrib_ptr(&attrib);
        }
        self.vbo.buffer_data(
            data.len() * size_of::<Triangle<V>>(),
            data.as_ptr() as *const f32,
            GlStorageMode::Static,
        );
        self.vbo.unbind();
        self.vao.unbind();
    }
    fn get_data(&self) -> Self::Data {
        let mut data: Self::Data = Vec::with_capacity(self.n_tris as usize);
        self.vbo.bind();
        unsafe {
            data.set_len(self.n_tris as usize);
            self.vbo.get_buffer_sub_data(
                0,
                self.n_tris as usize * size_of::<Triangle<V>>(),
                data.as_mut_ptr() as *mut f32,
            );
        }
        self.vbo.unbind();
        data
    }
}
impl<V: VertexBase> TriGPUBuffer<V> {
    pub fn n_tris(&self) -> i32 {
        self.n_tris
    }
}
