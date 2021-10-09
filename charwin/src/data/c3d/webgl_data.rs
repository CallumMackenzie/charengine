use crate::data::c3d::{TriCPUBuffer, Triangle, VertexBase};
use crate::data::{CPUBuffer, GPUBuffer, GPUShaderBase};
use crate::platform::Window;
use charmath::linear::matrix::{Mat2F, Mat4F, MatrixBase};
use charmath::linear::vector::{
    Vec2, Vec2f32, Vec2i32, Vec3, Vec3f32, Vec3i32, Vec4, Vec4f32, Vec4i32,
};
use js_sys::Float32Array;
use std::marker::PhantomData;
use std::mem::size_of;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log_string(a: &str);
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn js_warn_string(a: &str);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn js_err_string(a: &str);
}

#[allow(dead_code)]
pub struct WebGlTriGPUBuffer<V: VertexBase> {
    vao: Option<WebGlVertexArrayObject>,
    vbo: Option<WebGlBuffer>,
    n_tris: usize,
    context: Arc<Mutex<WebGl2RenderingContext>>,
    phantom: PhantomData<V>,
}
impl<V: VertexBase> WebGlTriGPUBuffer<V> {
    pub fn bind_vbo(&self) {
        self.context
            .lock()
            .unwrap()
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vbo.as_ref());
    }
    pub fn bind_vao(&self) {
        self.context
            .lock()
            .unwrap()
            .bind_vertex_array(self.vao.as_ref());
    }
    pub fn unbind_vao(&self) {
        self.context.lock().unwrap().bind_vertex_array(None);
    }
    pub fn unbind_vbo(&self) {
        self.context
            .lock()
            .unwrap()
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
    }
    pub fn set_attrib_ptr(&self, index: usize, size: usize, step: usize, offset: usize) {
        self.bind_vao();
        self.bind_vbo();
        self.context.lock().unwrap().vertex_attrib_pointer_with_i32(
            index as u32,
            size as i32,
            WebGl2RenderingContext::FLOAT,
            false,
            step as i32,
            offset as i32,
        );
        self.context
            .lock()
            .unwrap()
            .enable_vertex_attrib_array(index as u32);
        self.unbind_vbo();
        self.unbind_vao();
    }
    pub fn remove_attrib_ptr(&self, index: u32) {
        self.bind_vao();
        self.bind_vbo();
        if self.vao != None {
            self.context
                .lock()
                .unwrap()
                .disable_vertex_attrib_array(index);
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
impl<V: VertexBase> GPUBuffer for WebGlTriGPUBuffer<V> {
    type Data = Vec<Triangle<V>>;
    type CPUType = TriCPUBuffer<V>;
    fn new(win: &mut Window) -> Self {
        Self {
            vao: None,
            vbo: None,
            n_tris: 0,
            context: win.get_context_arc(),
            phantom: PhantomData,
        }
    }
    fn set_data(&mut self, data: &Self::Data) {
        if self.vbo == None {
            self.vbo = self.context.lock().unwrap().create_buffer();
            if self.vbo == None {
                js_err_string(&"Failed to create WebGL buffer.");
                panic!();
            }
        }
        self.bind_vbo();
        let f32_data = TriCPUBuffer::from_data(data).to_f32_array();
        unsafe {
            let positions_array_buf_view = Float32Array::view(&f32_data);
            if self.n_tris == data.len() {
                self.context
                    .lock()
                    .unwrap()
                    .buffer_sub_data_with_i32_and_array_buffer_view(
                        WebGl2RenderingContext::ARRAY_BUFFER,
                        0,
                        &positions_array_buf_view,
                    );
            } else {
                self.context
                    .lock()
                    .unwrap()
                    .buffer_data_with_array_buffer_view(
                        WebGl2RenderingContext::ARRAY_BUFFER,
                        &positions_array_buf_view,
                        WebGl2RenderingContext::STATIC_DRAW,
                    );
            }
        }
        self.n_tris = data.len();
        self.unbind_vbo();
    }
    fn get_data(&self) -> Self::Data {
        let positions_array_buf_view =
            Float32Array::new_with_length((self.n_tris * 3 * V::float_size()) as u32);
        self.bind_vbo();
        self.context
            .lock()
            .unwrap()
            .get_buffer_sub_data_with_i32_and_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                0,
                &positions_array_buf_view,
            );
        self.unbind_vbo();
        TriCPUBuffer::from_f32_array(&positions_array_buf_view.to_vec()).get_data()
    }
}
impl<V: VertexBase> Drop for WebGlTriGPUBuffer<V> {
    fn drop(&mut self) {
        let gl = self.context.lock().unwrap();
        if self.vbo != None {
            gl.delete_buffer(self.vbo.as_ref());
        }
        if self.vao != None {
            gl.delete_vertex_array(self.vao.as_ref());
        }
    }
}
pub struct WebGPUShader {
    program: Option<WebGlProgram>,
    context: Arc<Mutex<WebGl2RenderingContext>>,
}
impl WebGPUShader {
    fn gsl(&self, name: &str) -> Option<WebGlUniformLocation> {
        self.context
            .lock()
            .unwrap()
            .get_uniform_location(self.program.as_ref().unwrap(), name)
    }
    fn compile_shader(
        context: &Arc<Mutex<WebGl2RenderingContext>>,
        src: &str,
        s_type: u32,
    ) -> WebGlShader {
        let gl = context.lock().unwrap();
        let shader = gl
            .create_shader(s_type)
            .ok_or_else(|| {
                js_err_string(&"WebGL unable to create shader.");
                panic!();
            })
            .unwrap();
        gl.shader_source(&shader, src);
        gl.compile_shader(&shader);
        if !gl
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(true)
        {
            js_err_string(
                &gl.get_shader_info_log(&shader)
                    .unwrap_or_else(|| String::from("WebGL unknown error creating shader.")),
            );
            panic!();
        }
        shader
    }
}
impl GPUShaderBase for WebGPUShader {
    fn new(win: &Window) -> Self {
        Self {
            program: None,
            context: win.get_context_arc(),
        }
    }
    fn compile(&mut self, vsrc: &str, fsrc: &str) {
        let program = self
            .context
            .lock()
            .unwrap()
            .create_program()
            .ok_or_else(|| {
                js_err_string(&"WebGL unable to create program.");
                panic!();
            })
            .unwrap();
        let (vs, fs) = (
            Self::compile_shader(&self.context, vsrc, WebGl2RenderingContext::VERTEX_SHADER),
            Self::compile_shader(&self.context, fsrc, WebGl2RenderingContext::FRAGMENT_SHADER),
        );
        let gl = self.context.lock().unwrap();
        gl.attach_shader(&program, &vs);
        gl.attach_shader(&program, &fs);
        gl.link_program(&program);
        gl.delete_shader(Some(&vs));
        gl.delete_shader(Some(&fs));
        if !gl
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(true)
        {
            js_err_string(
                &gl.get_program_info_log(&program)
                    .unwrap_or_else(|| String::from("WebGL unknown error creating program.")),
            );
            panic!();
        }
        self.program = Some(program);
    }
    fn use_shader(&self) {
        self.context
            .lock()
            .unwrap()
            .use_program(self.program.as_ref());
    }
    fn draw(&self, n_tris: i32) {
        self.context
            .lock()
            .unwrap()
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, n_tris * 3);
    }
    fn set_4f32(&self, name: &str, v: &Vec4f32) {
        let loc = self.gsl(name);
        self.context.lock().unwrap().uniform4f(
            loc.as_ref(),
            v.get_x(),
            v.get_y(),
            v.get_z(),
            v.get_w(),
        );
    }
    fn set_3f32(&self, name: &str, v: &Vec3f32) {
        let loc = self.gsl(name);
        self.context
            .lock()
            .unwrap()
            .uniform3f(loc.as_ref(), v.get_x(), v.get_y(), v.get_z());
    }
    fn set_2f32(&self, name: &str, v: &Vec2f32) {
        let loc = self.gsl(name);
        self.context
            .lock()
            .unwrap()
            .uniform2f(loc.as_ref(), v.get_x(), v.get_y());
    }
    fn set_1f32(&self, name: &str, v: f32) {
        let loc = self.gsl(name);
        self.context.lock().unwrap().uniform1f(loc.as_ref(), v);
    }
    fn set_4i32(&self, name: &str, v: &Vec4i32) {
        let loc = self.gsl(name);
        self.context.lock().unwrap().uniform4i(
            loc.as_ref(),
            v.get_x(),
            v.get_y(),
            v.get_z(),
            v.get_w(),
        );
    }
    fn set_3i32(&self, name: &str, v: &Vec3i32) {
        let loc = self.gsl(name);
        self.context
            .lock()
            .unwrap()
            .uniform3i(loc.as_ref(), v.get_x(), v.get_y(), v.get_z());
    }
    fn set_2i32(&self, name: &str, v: &Vec2i32) {
        let loc = self.gsl(name);
        self.context
            .lock()
            .unwrap()
            .uniform2i(loc.as_ref(), v.get_x(), v.get_y());
    }
    fn set_1i32(&self, name: &str, v: i32) {
        let loc = self.gsl(name);
        self.context.lock().unwrap().uniform1i(loc.as_ref(), v);
    }
    fn set_mat4f32(&self, name: &str, v: &Mat4F) {
        let loc = self.gsl(name);
        self.context
            .lock()
            .unwrap()
            .uniform_matrix4fv_with_f32_array(loc.as_ref(), false, &v.flatten());
    }
    fn set_mat2f32(&self, name: &str, v: &Mat2F) {
        let loc = self.gsl(name);
        self.context
            .lock()
            .unwrap()
            .uniform_matrix2fv_with_f32_array(loc.as_ref(), false, &v.flatten());
    }
}
impl Drop for WebGPUShader {
    fn drop(&mut self) {
        if self.program != None {
            self.context
                .lock()
                .unwrap()
                .delete_program(self.program.as_ref());
        }
    }
}
