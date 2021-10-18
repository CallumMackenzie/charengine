use crate::char_panic;
use crate::platform::{Buffer, Program, Shader, Texture2D, VertexArray, Window};
use crate::window::*;
use charmath::linear::matrix::{Mat2F, Mat2f32, Mat4F, Mat4f32, MatrixBase};
use charmath::linear::vector::{
    Vec2, Vec2f32, Vec2i32, Vec3, Vec3f32, Vec3i32, Vec4, Vec4f32, Vec4i32,
};
use image::{ColorType, DynamicImage, GenericImage, GenericImageView};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Index, IndexMut};

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

pub trait DataBuffer: Sized {
    type Data: Sized;
    type IndexType: Sized;
    fn set_data(&mut self, data: &Self::Data);
    fn sub_data(&mut self, start: Self::IndexType, len: Self::IndexType, data: &Self::Data);
    fn get_data(&self) -> Self::Data;
    fn get_sub_data(&self, start: Self::IndexType, len: Self::IndexType) -> Self::Data;
    fn len(&self) -> Self::IndexType;

    fn to_data(self) -> Self::Data {
        self.get_data()
    }
}
pub trait CPUBuffer: DataBuffer {
    type GPUType: DataBuffer<Data = Self::Data, IndexType = Self::IndexType> + GPUBuffer;

    fn new() -> Self;
    fn from_data(data: &Self::Data) -> Self {
        let mut ret = Self::new();
        ret.set_data(data);
        ret
    }
    fn get_gpu_buffer(&self, win: &mut Window) -> Self::GPUType {
        Self::GPUType::from_data(win, &self.get_data())
    }
    fn to_gpu_buffer(self, win: &mut Window) -> Self::GPUType {
        Self::GPUType::from_data(win, &self.to_data())
    }
}

pub trait GPUBuffer: DataBuffer {
    type CPUType: DataBuffer<Data = Self::Data, IndexType = Self::IndexType> + CPUBuffer;
    fn new(win: &mut Window) -> Self;
    fn from_data(win: &mut Window, data: &Self::Data) -> Self {
        let mut ret = Self::new(win);
        ret.set_data(data);
        ret
    }
    fn get_cpu_buffer(&self) -> Self::CPUType {
        Self::CPUType::from_data(&self.get_data())
    }
    fn to_cpu_buffer(self) -> Self::CPUType {
        Self::CPUType::from_data(&self.to_data())
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
impl<V: VertexBase> DataBuffer for TriCPUBuffer<V> {
    type Data = Vec<Triangle<V>>;
    type IndexType = usize;
    fn set_data(&mut self, data: &Self::Data) {
        self.tris.clear();
        for i in 0..data.len() {
            self.tris.push(data[i]);
        }
    }
    fn sub_data(&mut self, start: usize, len: usize, data: &Self::Data) {
        if start + len > self.tris.len() {
            char_panic!("CPUTriBuffer.sub_data: Start is too far or len is too long.");
        }
        if data.len() < len {
            char_panic!("CPUTriBuffer.sub_data: Data length is smaller than input length.");
        }
        for i in 0..len {
            self.tris[i + start] = data[i];
        }
    }
    fn get_sub_data(&self, start: usize, len: usize) -> Self::Data {
        if start + len > self.tris.len() {
            char_panic!("CPUTriBuffer.get_sub_data: Start is too far or len is too long.");
        }
        let mut ret = Vec::with_capacity(len);
        for i in 0..len {
            ret.push(self.tris[start + i]);
        }
        ret
    }
    fn to_data(self) -> Self::Data {
        self.tris
    }
    fn len(&self) -> usize {
        self.tris.len()
    }
    fn get_data(&self) -> Self::Data {
        self.get_sub_data(0, self.len())
    }
}
impl<V: VertexBase> CPUBuffer for TriCPUBuffer<V> {
    type GPUType = TriGPUBuffer<V>;
    fn new() -> Self {
        TriCPUBuffer { tris: Vec::new() }
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

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct GPUShader {
    prog: Program,
}
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl GPUShader {
    #[cfg_attr(target_family = "wasm", wasm_bindgen(constructor))]
    pub fn new(w: &Window) -> Self {
        Self {
            prog: Program::new(w),
        }
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = compile))]
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
                    char_panic!("Error linking program: \n{}", err);
                }
            }
            (Err(vs), Err(fs)) => {
                char_panic!(
                    "Error compiling vertex and fragment shaders: \n{}\n\n{}",
                    vs,
                    fs
                );
            }
            (Err(vs), Ok(_)) => {
                char_panic!("Error compiling vertex shader: \n{}", vs);
            }
            (Ok(_), Err(fs)) => {
                char_panic!("Error compiling framgnet shader: \n{}", fs);
            }
        }
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = use))]
    pub fn use_shader(&self) {
        self.prog.bind();
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = draw))]
    pub fn draw(&self, nt: i32) {
        self.draw_from(0, nt);
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = drawFrom))]
    pub fn draw_from(&self, start: i32, n_tris: i32) {
        self.prog
            .draw_arrays(GlDrawMode::Triangles, start, n_tris * 3);
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = fromSources))]
    pub fn from_sources(w: &Window, v: &str, f: &str) -> Self {
        let ret = Self::new(w);
        ret.compile(w, v, f);
        ret
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setVec4f))]
    pub fn set_vec4f(&self, name: &str, vec: &Vec4f32) {
        self.prog
            .uniform_4f(&self.prog.shader_loc(name), vec.as_tuple());
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setVec3f))]
    pub fn set_vec3f(&self, name: &str, vec: &Vec3f32) {
        self.prog
            .uniform_3f(&self.prog.shader_loc(name), vec.as_tuple());
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setVec2f))]
    pub fn set_vec2f(&self, name: &str, vec: &Vec2f32) {
        self.prog
            .uniform_2f(&self.prog.shader_loc(name), vec.as_tuple());
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setFloat))]
    pub fn set_float(&self, name: &str, vec: f32) {
        self.prog.uniform_1f(&self.prog.shader_loc(name), vec);
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setVec4i))]
    pub fn set_vec4i(&self, name: &str, vec: &Vec4i32) {
        self.prog
            .uniform_4i(&self.prog.shader_loc(name), vec.as_tuple());
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setVec3i))]
    pub fn set_vec3i(&self, name: &str, vec: &Vec3i32) {
        self.prog
            .uniform_3i(&self.prog.shader_loc(name), vec.as_tuple());
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setVec2i))]
    pub fn set_vec2i(&self, name: &str, vec: &Vec2i32) {
        self.prog
            .uniform_2i(&self.prog.shader_loc(name), vec.as_tuple());
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setInt))]
    pub fn set_int(&self, name: &str, vec: i32) {
        self.prog.uniform_1i(&self.prog.shader_loc(name), vec);
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setMat4f))]
    pub fn set_mat4f32(&self, name: &str, mat: &Mat4f32) {
        self.prog
            .uniform_mat4f(&self.prog.shader_loc(name), &mat.flatten());
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(js_name = setMat2f))]
    pub fn set_mat2f32(&self, name: &str, mat: &Mat2f32) {
        self.prog
            .uniform_mat2f(&self.prog.shader_loc(name), &mat.flatten());
    }
}
impl GPUShader {
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
    type CPUType = TriCPUBuffer<V>;
    fn new(win: &mut Window) -> Self {
        Self {
            vbo: Buffer::new(win, GlBufferType::ArrayBuffer),
            vao: VertexArray::new(win),
            n_tris: 0,
            phantom: PhantomData,
        }
    }
}
impl<V: VertexBase> DataBuffer for TriGPUBuffer<V> {
    type Data = Vec<Triangle<V>>;
    type IndexType = usize;
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
    fn sub_data(&mut self, start: usize, len: usize, data: &Self::Data) {
        if data.len() < len {
            char_panic!("TriGPUBuffer.sub_data: Input data size too small.");
        }
        if start + len > self.n_tris as usize * V::float_size() * 3 {
            char_panic!("TriGPUBuffer.sub_data: Input range will not fit inside butter.");
        }
        self.vbo.bind();
        self.vbo.buffer_sub_data(
            start * size_of::<V>() * 3,
            len * size_of::<V>() * 3,
            data.as_ptr() as *const f32,
        );
        self.vbo.unbind();
    }
    fn get_sub_data(&self, start: usize, len: usize) -> Self::Data {
        let mut recv = Self::Data::with_capacity(len);
        self.vbo.bind();
        unsafe {
            recv.set_len(len as usize);
            self.vbo.get_buffer_sub_data(
                start * size_of::<Triangle<V>>(),
                len * size_of::<Triangle<V>>(),
                recv.as_mut_ptr() as *mut f32,
            );
        }
        recv.shrink_to_fit();
        self.vbo.unbind();
        recv
    }
    fn len(&self) -> usize {
        self.n_tris as usize
    }
    fn get_data(&self) -> Self::Data {
        self.get_sub_data(0, self.len())
    }
}
impl<V: VertexBase> TriGPUBuffer<V> {
    pub fn n_tris(&self) -> i32 {
        self.n_tris
    }
}

impl DataBuffer for DynamicImage {
    type Data = DynamicImage;
    type IndexType = (u32, u32);
    fn set_data(&mut self, data: &Self::Data) {
        self.copy_from(&data.view(0, 0, data.width(), data.height()), 0, 0)
            .unwrap_or_else(|e| {
                char_panic!("Could not copy image data: {:?}.", e);
            })
    }
    fn sub_data(&mut self, start: Self::IndexType, len: Self::IndexType, data: &Self::Data) {
        self.copy_from(&data.view(0, 0, len.0, len.1), start.0, start.1)
            .unwrap_or_else(|e| {
                char_panic!("Could not copy image data: {:?}.", e);
            });
    }
    fn get_sub_data(&self, start: Self::IndexType, len: Self::IndexType) -> Self::Data {
        self.crop_imm(start.0, start.1, len.0, len.1)
    }
    fn len(&self) -> Self::IndexType {
        (self.width(), self.height())
    }
    fn get_data(&self) -> Self::Data {
        self.get_sub_data((0, 0), self.len())
    }
}
impl CPUBuffer for DynamicImage {
    type GPUType = GPUTexture;
    fn new() -> Self {
        Self::new_rgba8(0, 0)
    }
}
pub trait DynamicImageColorable {
    fn solid_color(col: [u8; 4]) -> Self;
}
impl DynamicImageColorable for DynamicImage {
    fn solid_color(col: [u8; 4]) -> Self {
        let mut default_image = DynamicImage::new_rgba8(1, 1);
        if let Some(img) = default_image.as_mut_rgba8() {
            img.put_pixel(0, 0, image::Rgba::from(col));
        }
        default_image
    }
}
trait GlImageFormatConvertable {
    fn gl_image_fmt(&self) -> GlInternalTextureFormat;
    fn gl_pixel_fmt(&self) -> GlImagePixelFormat;
    fn gl_pixel_type(&self) -> GlImagePixelType;
    fn pixel_byte_count(&self) -> usize;
}
impl GlImageFormatConvertable for DynamicImage {
    fn gl_image_fmt(&self) -> GlInternalTextureFormat {
        use ColorType::*;
        use GlInternalTextureFormat::*;
        match self.color() {
            L8 => R8,
            L16 => R16,
            La8 => RG8,
            La16 => RG16,
            Rgb8 => RGB8,
            Rgb16 => RGB12,
            Rgba8 => RGBA8,
            Rgba16 => RGBA16,
            _ => {
                char_panic!("Cannot find valid gl texture format from image format.");
            }
        }
    }
    fn gl_pixel_fmt(&self) -> GlImagePixelFormat {
        use ColorType::*;
        use GlImagePixelFormat::*;
        match self.color() {
            L8 | L16 => Red,
            La8 | La16 => RG,
            Rgb8 | Rgb16 => RGB,
            Rgba8 | Rgba16 => RGBA,
            Bgr8 => BGR,
            Bgra8 => BGRA,
            _ => {
                char_panic!("Cannot find valid gl color pixel format from image pixel format.");
            }
        }
    }
    fn gl_pixel_type(&self) -> GlImagePixelType {
        use ColorType::*;
        use GlImagePixelType::*;
        match self.color() {
            L8 | La8 | Rgb8 | Rgba8 | Bgr8 | Bgra8 => UnsignedByte,
            L16 | La16 | Rgb16 | Rgba16 => UnsignedShort,
            _ => {
                char_panic!("Cannot find valid gl color pixel type from image pixel type.");
            }
        }
    }
    fn pixel_byte_count(&self) -> usize {
        self.color().bytes_per_pixel() as usize
    }
}

pub struct GPUTexture {
    pub tex: Texture2D,
    pub size: (u32, u32),
}
impl GPUTexture {
	pub fn set_data_mips(&mut self, data: &DynamicImage, mips: Option<u32>) {
        use DynamicImage::*;
        self.size = (data.width(), data.height());
        self.tex.bind();
        self.tex.set_texture(
            match data {
                ImageLuma8(img) => img.as_raw().as_ptr() as *const u8,
                ImageLumaA8(img) => img.as_raw().as_ptr() as *const u8,
                ImageRgb8(img) => img.as_raw().as_ptr() as *const u8,
                ImageRgba8(img) => img.as_raw().as_ptr() as *const u8,
                ImageBgr8(img) => img.as_raw().as_ptr() as *const u8,
                ImageBgra8(img) => img.as_raw().as_ptr() as *const u8,
                ImageLuma16(img) => img.as_raw().as_ptr() as *const u8,
                ImageLumaA16(img) => img.as_raw().as_ptr() as *const u8,
                ImageRgb16(img) => img.as_raw().as_ptr() as *const u8,
                ImageRgba16(img) => img.as_raw().as_ptr() as *const u8,
            },
            self.size.0,
            self.size.1,
            data.gl_image_fmt(),
            data.gl_pixel_fmt(),
            data.gl_pixel_type(),
            mips,
            data.pixel_byte_count(),
        );
        self.tex.unbind();
	}
}
impl DataBuffer for GPUTexture {
    type Data = DynamicImage;
    type IndexType = (u32, u32);
    fn set_data(&mut self, data: &Self::Data) {
		self.set_data_mips(data, None);
    }
    fn sub_data(&mut self, _: Self::IndexType, _: Self::IndexType, _: &Self::Data) {
        char_panic!("Cannot set data of GPU texture.");
    }
    fn get_sub_data(&self, _: Self::IndexType, _: Self::IndexType) -> Self::Data {
        char_panic!("Cannot get data of GPU texture.");
    }
    fn get_data(&self) -> Self::Data {
        self.get_sub_data((0, 0), (0, 0))
    }
    fn len(&self) -> Self::IndexType {
        self.size
    }
}
impl GPUBuffer for GPUTexture {
    type CPUType = DynamicImage;
    fn new(win: &mut Window) -> Self {
        Self {
            tex: Texture2D::new(win),
            size: (0, 0),
        }
    }
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub struct WebGlTriGPUBufferVertexV {
    buff: TriGPUBuffer<VertexV>,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl WebGlTriGPUBufferVertexV {
    #[wasm_bindgen(js_name = fromFloatArray)]
    pub fn from_float_array(win: &mut Window, arr: &[f32]) -> Self {
        Self {
            buff: TriCPUBuffer::<VertexV>::from_f32_array(arr).to_gpu_buffer(win),
        }
    }
    #[wasm_bindgen(js_name = bindVAO)]
    pub fn bind_vao(&self) {
        self.buff.vao.bind();
    }
    #[wasm_bindgen(js_name = nTris)]
    pub fn n_tris(&self) -> i32 {
        self.buff.n_tris()
    }
}
