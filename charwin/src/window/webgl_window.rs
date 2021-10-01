use crate::window::{AbstractWindow, AbstractWindowFactory, WindowCreateArgs, WindowEvent};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;

#[wasm_bindgen]
pub struct WebGlWindow {
    context: WebGl2RenderingContext,
    clear_mask: u32,
}

#[wasm_bindgen]
impl WebGlWindow {
    #[wasm_bindgen(constructor)]
    pub fn wcreate(args: &WindowCreateArgs) -> Self {
        Self::create(args)
    }
}

impl AbstractWindow for WebGlWindow {
    fn set_fullscreen(&mut self) {
        unimplemented!();
    }
    fn set_windowed(&mut self) {
        unimplemented!();
    }
    fn set_title(&mut self, _title: &str) {
        unimplemented!();
    }
    fn set_size(&mut self, _w: u32, _h: u32) {
        unimplemented!();
    }
    fn close(&mut self) {
        unimplemented!();
    }
    fn should_close(&mut self) -> bool {
        false
    }
    fn poll_events(&mut self) {}
    fn get_events(&mut self) -> Vec<WindowEvent> {
        Vec::<WindowEvent>::new()
    }
    fn swap_buffers(&mut self) {}
    fn set_clear_colour(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.context
            .clear_color(r as f32, g as f32, b as f32, a as f32);
    }
    fn clear(&mut self) {
        self.context.clear(self.clear_mask)
    }
}

impl AbstractWindowFactory for WebGlWindow {
    fn create(args: &WindowCreateArgs) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(&args.title).unwrap();
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        WebGlWindow {
            context: canvas
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap(),
            clear_mask: WebGl2RenderingContext::COLOR_BUFFER_BIT,
        }
    }
}
