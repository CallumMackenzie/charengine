use crate::data::{CPUBuffer, GPUBuffer};
use crate::platform::TriGPUBuffer;
use crate::platform::Window;
use charmath::linear::vector::{Vec2, Vec2F, Vec3, Vec3F};
use std::ops::{Index, IndexMut};

#[cfg(not(target_family = "wasm"))]
pub mod opengl_data;
#[cfg(target_family = "wasm")]
pub mod webgl_data;

pub fn get_triangle_data() -> Vec<Triangle<VertexVTN>> {
    vec![Triangle::from_verts(&[
        VertexVTN::from_point_uv_norm(
            &Vec3F::new(0.0, 0.0, 0.0),
            &Vec2F::new(0.0, 0.0),
            &Vec3F::new(0.0, 1.0, 0.0),
        ),
        VertexVTN::from_point_uv_norm(
            &Vec3F::new(1.0, 0.0, 0.0),
            &Vec2F::new(1.0, 0.0),
            &Vec3F::new(0.0, 1.0, 0.0),
        ),
        VertexVTN::from_point_uv_norm(
            &Vec3F::new(0.0, 0.0, 1.0),
            &Vec2F::new(0.0, 1.0),
            &Vec3F::new(0.0, 1.0, 0.0),
        ),
    ])]
}

pub trait VertexBase: Copy + Sized {
    fn new() -> Self;
    fn float_size() -> usize;

    fn get_point(&self) -> Vec3F;
    fn get_uv(&self) -> Vec2F;
    fn get_normal(&self) -> Vec3F;

    fn set_point(&mut self, p: &Vec3F);
    fn set_uv(&mut self, t: &Vec2F);
    fn set_normal(&mut self, n: &Vec3F);

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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VertexVTN {
    pub v: Vec3F,
    pub t: Vec2F,
    pub n: Vec3F,
}
impl VertexVTN {
    pub fn from_point(p: &Vec3F) -> Self {
        let mut ret = Self::new();
        ret.set_point(p);
        ret
    }
    pub fn from_point_uv(p: &Vec3F, t: &Vec2F) -> Self {
        let mut ret = Self::from_point(p);
        ret.set_uv(t);
        ret
    }
    pub fn from_point_uv_norm(p: &Vec3F, t: &Vec2F, n: &Vec3F) -> Self {
        let mut ret = Self::from_point_uv(p, t);
        ret.set_normal(n);
        ret
    }
}
impl VertexBase for VertexVTN {
    fn new() -> Self {
        Self {
            v: Vec3F::new(0f32, 0f32, 0f32),
            t: Vec2F::new(0f32, 0f32),
            n: Vec3F::new(0f32, 0f32, 0f32),
        }
    }
    fn float_size() -> usize {
        8usize
    }
    fn get_point(&self) -> Vec3F {
        self.v
    }
    fn get_uv(&self) -> Vec2F {
        self.t
    }
    fn get_normal(&self) -> Vec3F {
        self.n
    }
    fn set_point(&mut self, p: &Vec3F) {
        self.v = *p;
    }
    fn set_uv(&mut self, t: &Vec2F) {
        self.t = *t;
    }
    fn set_normal(&mut self, n: &Vec3F) {
        self.n = *n;
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

pub struct TriCPUBuffer<V: VertexBase> {
    tris: Vec<Triangle<V>>,
}
impl<V: VertexBase> TriCPUBuffer<V> {
    pub fn n_tris(&self) -> usize {
        self.tris.len()
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

pub type TriVTNCPUBuffer = TriCPUBuffer<VertexVTN>;
pub type TriVTNGPUBuffer = TriGPUBuffer<VertexVTN>;

pub trait TriNumConvertable<V: VertexBase> {
    fn self_from_tris(tris: &Vec<Triangle<V>>) -> Self;
    fn tris_from_self(arr: &Self) -> Vec<Triangle<V>>;
}
impl<V: VertexBase> TriNumConvertable<V> for Vec<f32> {
    fn self_from_tris(tris: &Vec<Triangle<V>>) -> Vec<f32> {
        let mut ret = Vec::with_capacity(tris.len() * 3 * V::float_size());
        for i in 0..tris.len() {
            for j in 0..3 {
                for float in tris[i].v[j].to_f32_array() {
                    ret.push(float);
                }
            }
        }
        ret
    }
    fn tris_from_self(arr: &Vec<f32>) -> Vec<Triangle<V>> {
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
        data
    }
}
