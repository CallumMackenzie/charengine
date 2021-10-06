use crate::data::c3d::{TriCPUBuffer, TriNumConvertable, Triangle, VertexBase};
use crate::data::GPUBuffer;
use crate::platform::Window;
use js_sys::Float32Array;
use std::marker::PhantomData;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log_string(a: &str);
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn js_warn_string(a: &str);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn js_err_string(a: &str);

    fn alert(a: &str);
}

#[allow(dead_code)]
pub struct WebGlTriGPUBuffer<V: VertexBase> {
    vao: Option<WebGlVertexArrayObject>,
    vbo: Option<WebGlBuffer>,
    n_tris: usize,
    phantom: PhantomData<V>,
}
impl<V: VertexBase> GPUBuffer for WebGlTriGPUBuffer<V> {
    type Data = Vec<Triangle<V>>;
    type CPUType = TriCPUBuffer<V>;
    fn new() -> Self {
        Self {
            vao: None,
            vbo: None,
            n_tris: 0,
            phantom: PhantomData,
        }
    }
    fn set_data(&mut self, win: &mut Window, data: &Self::Data) {
        let gl = win.get_context_ref();
        if let None = self.vbo {
            self.vbo = gl.create_buffer();
            if let None = self.vbo {
                js_err_string(&"Failed to create WebGL buffer.");
                panic!();
            }
        }
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vbo.as_ref());
        if self.n_tris == data.len() {
            unsafe {
                let positions_array_buf_view =
                    Float32Array::view(&Vec::<f32>::self_from_tris(data));
                gl.buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &positions_array_buf_view,
                );
            }
        } else {
            unsafe {
                let positions_array_buf_view =
                    Float32Array::view(&Vec::<f32>::self_from_tris(data));
                gl.buffer_data_with_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    &positions_array_buf_view,
                    WebGl2RenderingContext::STATIC_DRAW,
                );
            }
        }
        self.n_tris = data.len();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
    }
    fn get_data(&self, win: &mut Window) -> Self::Data {
        let gl = win.get_context_ref();
        let positions_array_buf_view =
            Float32Array::new_with_length((self.n_tris * 3 * V::float_size()) as u32);
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vbo.as_ref());
        gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0,
            &positions_array_buf_view,
        );
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        Vec::<f32>::tris_from_self(&positions_array_buf_view.to_vec())
    }
}
impl<V: VertexBase> Drop for WebGlTriGPUBuffer<V> {
    fn drop(&mut self) {}
}
