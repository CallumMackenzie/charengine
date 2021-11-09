use crate::char_panic;
use crate::data::{CPUBuffer, DynamicImageColorable, GPUTexture};
use crate::input::{Key, MouseButton};
use crate::platform::{Context, Window};
use crate::state::State;
use crate::window::*;
use image::DynamicImage;
use js_sys::{Float32Array, Uint8Array};
use std::cell::RefCell;
use std::collections::HashSet;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Event, HtmlCanvasElement, HtmlImageElement, KeyboardEvent, MouseEvent, WebGl2RenderingContext,
    WebGlBuffer as JsSysWebGlBuffer, WebGlProgram as JsSysWebGlProgram,
    WebGlShader as JsSysWebGlShader, WebGlTexture as JsSysWebGlTexture,
    WebGlUniformLocation as JsSysWebGlUniformLocation,
    WebGlVertexArrayObject as JsSysWebGlVertexArray, WheelEvent,
};

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

#[wasm_bindgen]
pub struct WebWindowEventSet {
    events: Vec<WindowEvent>,
}
#[wasm_bindgen]
impl WebWindowEventSet {
    #[wasm_bindgen(constructor)]
    pub fn wnew() -> WebWindowEventSet {
        WebWindowEventSet { events: Vec::new() }
    }
}
impl WebWindowEventSet {
    pub fn get_events(&self) -> Vec<WindowEvent> {
        let mut ret = Vec::new();
        for i in 0..self.events.len() {
            ret.push(self.events[i]);
        }
        ret
    }
}
struct WebEventListener {
    events: Vec<WindowEvent>,
    window_listeners: Vec<WindowEventListener>,
    canvas_listeners: Vec<CanvasEventListener>,
}
impl WebEventListener {
    pub fn add_window_listener(&mut self, ls: WindowEventListener) {
        self.window_listeners.push(ls);
    }
    pub fn add_canvas_listener(&mut self, ls: CanvasEventListener) {
        self.canvas_listeners.push(ls);
    }
    pub fn add_event(&mut self, state: WindowEvent) {
        self.events.push(state);
    }
    pub fn clear(&mut self) {
        self.events.clear();
    }
    pub fn get_events(&mut self) -> Vec<WindowEvent> {
        let mut ret = Vec::<WindowEvent>::new();
        for i in 0..self.events.len() {
            ret.push(self.events[i]);
        }
        ret
    }
    pub fn clear_listeners(&mut self) {
        self.window_listeners.clear();
        self.canvas_listeners.clear();
    }
}

#[wasm_bindgen]
pub struct WebGlWindow {
    context: Arc<Mutex<WebGl2RenderingContext>>,
    canvas: Arc<Mutex<HtmlCanvasElement>>,
    should_close: bool,
    event_listener: Option<Arc<Mutex<WebEventListener>>>,
    events: Vec<WindowEvent>,
}

#[wasm_bindgen]
impl WebGlWindow {
    #[wasm_bindgen(constructor)]
    pub fn wcreate(args: &WindowCreateArgs) -> Self {
        Self::create(args)
    }
    #[wasm_bindgen(js_name = setClearColour)]
    pub fn wset_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.set_clear_colour(r, g, b, a);
    }
    #[wasm_bindgen(js_name = clear)]
    pub fn wclear(&mut self, int_mask: i32) {
        let mut mask = Vec::<GlClearMask>::with_capacity(3);
        if int_mask & (GlClearMask::Color as i32) == 1 {
            mask.push(GlClearMask::Color);
        }
        if int_mask & (GlClearMask::Depth as i32) == 1 {
            mask.push(GlClearMask::Depth);
        }
        if int_mask & (GlClearMask::Stencil as i32) == 1 {
            mask.push(GlClearMask::Stencil);
        }
        self.clear(&mask);
    }
    #[wasm_bindgen(js_name = getEvents)]
    pub fn wget_event_set(&mut self) -> WebWindowEventSet {
        WebWindowEventSet {
            events: self.get_events(),
        }
    }
    #[wasm_bindgen(js_name = pollEvents)]
    pub fn wpoll_events(&mut self) {
        self.poll_events();
    }
    #[wasm_bindgen(js_name = destroy)]
    pub fn wdestroy(&mut self) {
        if let Some(mutex) = self.event_listener.as_ref() {
            if let Ok(mut listener) = mutex.lock() {
                listener.clear_listeners();
            } else {
                char_panic!("JS: Could not destroy window: mutex posioned.");
            }
        } else {
            js_warn_string(&"JS: No window event listener found to destroy.");
        }
    }
    #[wasm_bindgen(js_name = setResolution)]
    pub fn wset_resolution(&mut self, wid: f64, hei: f64) {
        self.set_resolution((wid as i32, hei as i32));
    }
    #[wasm_bindgen(js_name = setSize)]
    pub fn wset_size(&mut self, wid: f64, hei: f64) {
        self.set_size((wid as i32, hei as i32));
    }
}

impl AbstractWindow for WebGlWindow {
    fn get_gl_context(&mut self) -> Context {
        Context::new(self)
    }
    fn set_fullscreen(&mut self) {
        // Does nothing on WASM
    }
    fn set_windowed(&mut self) {
        // Does nothing on WASM
    }
    fn set_title(&mut self, title: &str) {
        self.canvas.lock().unwrap().set_id(title);
    }
    fn set_size(&mut self, sz: (i32, i32)) {
        self.canvas.lock().unwrap().set_width(sz.0 as u32);
        self.canvas.lock().unwrap().set_height(sz.1 as u32);
    }
    fn close(&mut self) {
        self.should_close = true;
    }
    fn should_close(&mut self) -> bool {
        self.should_close
    }
    fn poll_events(&mut self) {
        if let Some(listener) = self.event_listener.as_ref() {
            self.events.clear();
            let events = listener.lock().unwrap().get_events();
            for i in 0..events.len() {
                self.events.push(events[i]);
            }
            listener.lock().unwrap().clear();
        }
    }
    fn get_events(&mut self) -> Vec<WindowEvent> {
        let mut ret = Vec::new();
        for i in 0..self.events.len() {
            ret.push(self.events[i]);
        }
        ret
    }
    fn swap_buffers(&mut self) {
        // Does nothing on WASM
    }
    fn get_size(&self) -> (i32, i32) {
        let bounding_rect = self.canvas.lock().unwrap().get_bounding_client_rect();
        (bounding_rect.width() as i32, bounding_rect.height() as i32)
    }
    fn get_pos(&self) -> (i32, i32) {
        let bounding_rect = self.canvas.lock().unwrap().get_bounding_client_rect();
        (bounding_rect.x() as i32, bounding_rect.y() as i32)
    }
    fn load_texture_rgba(&mut self, path: &str, mipmaps: Option<u32>) -> Arc<Mutex<GPUTexture>> {
        let tex = Arc::new(Mutex::new(
            DynamicImage::solid_color([0xff, 0x80, 0xff, 0xff]).to_gpu_buffer(self),
        ));
        let image = Arc::new(Mutex::new(HtmlImageElement::new().unwrap_or_else(|e| {
            char_panic!("Could not create new HtmlImageElement: {:?}.", e);
        })));
        image.lock().unwrap().set_cross_origin(Some("anonymous"));
        let f = Rc::new(RefCell::new(None));
        let g: Rc<RefCell<Option<Closure<_>>>> = f.clone();
        let tex_arc = Arc::clone(&tex);
        let image_arc = Arc::clone(&image);
        let context_arc = self.get_context_arc();
        let path_string = path.to_string();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if let Ok(mut tex) = tex_arc.lock() {
                if let Ok(image) = image_arc.lock() {
                    tex.size = (image.width(), image.height());
                    tex.tex.bind();
                    let mips = if u32::is_power_of_two(image.width())
                        && u32::is_power_of_two(image.height())
                    {
                        mipmaps
                    } else {
                        if mipmaps != None {
                            js_warn_string(&format!("Image ({}) improper size ({}x{}) to support specific mipmap level {}.", path_string, image.width(), image.height(), mipmaps.unwrap()));
                        }
                        None
                    };
                    context_arc
                        .lock()
                        .unwrap()
                        .tex_image_2d_with_u32_and_u32_and_html_image_element(
                            WebGl2RenderingContext::TEXTURE_2D,
                            mips.unwrap_or_else(|| 0) as i32,
                            WebGl2RenderingContext::RGBA as i32,
                            WebGl2RenderingContext::RGBA,
                            WebGl2RenderingContext::UNSIGNED_BYTE,
                            &image,
                        )
                        .unwrap_or_else(|e| {
                            js_err_string(&format!("WebGL: Could not texture image: {:?}", e));
                        });
                    tex.tex.set_params(mips);
                    tex.tex.unbind();
                } else {
                    js_err_string(&"Loading image element mutex poisoned.");
                }
            } else {
                js_err_string(&"Loading texture mutex poisoned.");
            }
            let _ = f.borrow_mut().take();
        }) as Box<dyn FnMut()>));
        image
            .lock()
            .unwrap()
            .set_onload(Some(g.borrow().as_ref().unwrap().as_ref().unchecked_ref()));
        image.lock().unwrap().set_src(path);
        tex
    }
}

impl AbstractWindowFactory for WebGlWindow {
    fn create(args: &WindowCreateArgs) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        if let Some(canvas) = document.get_element_by_id(&args.title) {
            if let Ok(canvas) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
                if let Ok(Some(context)) = canvas.get_context("webgl2") {
                    let mut window = WebGlWindow {
                        context: Arc::new(Mutex::new(
                            context.dyn_into::<WebGl2RenderingContext>().unwrap(),
                        )),
                        canvas: Arc::new(Mutex::new(canvas)),
                        should_close: false,
                        event_listener: None,
                        events: Vec::new(),
                    };
                    window.add_event_listeners();
                    window
                        .event_listener
                        .as_ref()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .add_event(WindowEvent::Size(window.get_size().0, window.get_size().1));
                    window
                } else {
                    char_panic!("WebGL: Platform may not support WebGL2.");
                }
            } else {
                char_panic!("DOM: Element ID \"{}\" is not a canvas.", &args.title);
            }
        } else {
            char_panic!(
                "DOM: Could not find canvas \"{}\" on document.",
                &args.title
            );
        }
    }
}

impl WebGlWindow {
    pub fn render_loop<S: State, E: EventManager>(mut self, mut state: S, mut manager: E) {
        if let Some(_) = web_sys::window() {
            let f = Rc::new(RefCell::new(None));
            let g: Rc<RefCell<Option<Closure<_>>>> = f.clone();
            let mut state_initialized = false;
            let mut last_frame = js_sys::Date::now();
            *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                if !state_initialized {
                    let rect = self.canvas.lock().unwrap().get_bounding_client_rect();
                    self.events
                        .push(WindowEvent::Size(rect.width() as i32, rect.height() as i32));
                    manager.process_events(&self.get_events());
                    state.initialize(&mut self, &mut manager);
                    state_initialized = true;
                } else {
                    self.poll_events();
                    manager.process_events(&self.get_events());
                }
                let update_res = state.update(
                    &mut self,
                    &mut manager,
                    (js_sys::Date::now() - last_frame) / 1000f64,
                );
                if update_res == 0 && !self.should_close {
                    last_frame = js_sys::Date::now();
                    let _ = web_sys::window().unwrap().request_animation_frame(
                        f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                    );
                } else {
                    state.destroy(&mut self, &mut manager, update_res);
                    let _ = f.borrow_mut().take();
                }
            }) as Box<dyn FnMut()>));
            web_sys::window()
                .unwrap()
                .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .unwrap();
        }
    }
    pub fn get_context_arc(&self) -> Arc<Mutex<WebGl2RenderingContext>> {
        Arc::clone(&self.context)
    }
}
impl Drop for WebGlWindow {
    fn drop(&mut self) {
        if let Some(mutex) = self.event_listener.as_ref() {
            if let Ok(mut listener) = mutex.lock() {
                listener.clear_listeners();
            } else {
                char_panic!("DOM: Cannot remove event listeners: mutex poisoned.");
            }
        }
        if let Ok(mut _context) = self.context.lock() {
            // Drop context
        } else {
            char_panic!("DOM: Cannot drop canvas context: mutex poisoned.");
        }
    }
}

struct CanvasEventListener {
    element: Arc<Mutex<HtmlCanvasElement>>,
    name: &'static str,
    cb: Closure<dyn FnMut(Event)>,
}
impl CanvasEventListener {
    fn new(
        element: Arc<Mutex<HtmlCanvasElement>>,
        name: &'static str,
        cb: Closure<dyn FnMut(Event)>,
    ) -> Self {
        element
            .lock()
            .unwrap()
            .add_event_listener_with_callback(name, cb.as_ref().unchecked_ref())
            .unwrap();
        Self { element, name, cb }
    }
}
impl Drop for CanvasEventListener {
    fn drop(&mut self) {
        self.element
            .lock()
            .unwrap()
            .remove_event_listener_with_callback(self.name, self.cb.as_ref().unchecked_ref())
            .unwrap();
    }
}

struct WindowEventListener {
    name: &'static str,
    cb: Closure<dyn FnMut(Event)>,
}
impl WindowEventListener {
    fn new(name: &'static str, cb: Closure<dyn FnMut(Event)>) -> Self {
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback(name, cb.as_ref().unchecked_ref())
            .unwrap();
        Self { name, cb }
    }
}
impl Drop for WindowEventListener {
    fn drop(&mut self) {
        web_sys::window()
            .unwrap()
            .remove_event_listener_with_callback(self.name, self.cb.as_ref().unchecked_ref())
            .unwrap();
    }
}
impl WebGlWindow {
    fn add_event_listeners(&mut self) {
        let listener = Arc::new(Mutex::new(WebEventListener {
            events: Vec::new(),
            window_listeners: Vec::new(),
            canvas_listeners: Vec::new(),
        }));
        self.add_keydown_listener(&listener);
        self.add_keyup_listener(&listener);
        self.add_keyheld_listener(&listener);
        self.add_mousedown_listener(&listener);
        self.add_mouseup_listener(&listener);
        self.add_mouseheld_listener(&listener);
        self.add_mousemove_listener(&listener);
        self.add_focus_listener(&listener);
        self.add_blur_listener(&listener);
        self.add_mouseenter_listener(&listener);
        self.add_mouseleave_listener(&listener);
        self.add_onclose_listener(&listener);
        self.add_resize_listener(&listener);
        self.add_scroll_listener(&listener);
        self.event_listener = Some(listener);
    }
    #[inline]
    fn add_event_listener(
        &mut self,
        mutex: &Arc<Mutex<WebEventListener>>,
        name: &'static str,
        cl: Closure<dyn FnMut(Event)>,
    ) {
        if let Ok(mut listener) = mutex.lock() {
            let canvas_arc = Arc::clone(&self.canvas);
            listener.add_canvas_listener(CanvasEventListener::new(canvas_arc, name, cl));
        } else {
            char_panic!(
                "DOM: Could not add canvas event listener: mutex poisoned: ({}).",
                name
            );
        }
    }
    #[inline]
    fn add_win_event_listener(
        mutex: &Arc<Mutex<WebEventListener>>,
        name: &'static str,
        cl: Closure<dyn FnMut(Event)>,
    ) {
        if let Ok(mut listener) = mutex.lock() {
            listener.add_window_listener(WindowEventListener::new(name, cl));
        } else {
            char_panic!(
                "DOM: Could not add window event listener: mutex poisoned: ({}).",
                name
            );
        }
    }
    #[inline]
    fn add_scroll_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            if let Ok(event) = e.dyn_into::<WheelEvent>() {
                state_clone
                    .lock()
                    .unwrap()
                    .add_event(WindowEvent::Scroll(event.delta_x(), event.delta_y()));
            }
        }) as Box<dyn FnMut(_)>);
        Self::add_win_event_listener(lstnr, "wheel", cl);
    }
    #[inline]
    fn add_resize_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let canvas = Arc::clone(&self.canvas);
        let cl = Closure::wrap(Box::new(move |_: Event| {
            let canvas_rect = canvas.lock().unwrap().get_bounding_client_rect();
            state_clone.lock().unwrap().add_event(WindowEvent::Size(
                canvas_rect.width() as i32,
                canvas_rect.height() as i32,
            ));
        }) as Box<dyn FnMut(_)>);
        Self::add_win_event_listener(lstnr, "resize", cl);
    }
    #[inline]
    fn add_onclose_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            if let Ok(mut state) = state_clone.lock() {
                state.add_event(WindowEvent::Close);
            } else {
                char_panic!("DOM: Could not add closing events: mutex posioned.");
            }
        }) as Box<dyn FnMut(_)>);
        Self::add_win_event_listener(lstnr, "beforeunload", cl);
    }
    #[inline]
    fn add_mouseleave_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        self.mouse_enter_listener_template(lstnr, "mouseleave", false);
    }
    #[inline]
    fn add_mouseenter_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        self.mouse_enter_listener_template(lstnr, "mouseenter", true);
    }
    #[inline]
    fn add_focus_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        self.focus_listener_template(lstnr, "focus", true);
    }
    #[inline]
    fn add_blur_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        self.focus_listener_template(lstnr, "blur", false);
    }
    #[inline]
    fn add_mousemove_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            if let Ok(event) = e.dyn_into::<MouseEvent>() {
                state_clone
                    .lock()
                    .unwrap()
                    .add_event(WindowEvent::CursorPosition(
                        event.offset_x() as f64,
                        event.offset_y() as f64,
                    ));
            }
        }) as Box<dyn FnMut(_)>);
        self.add_event_listener(lstnr, "mousemove", cl);
    }
    #[inline]
    fn add_keydown_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            if let Ok(event) = e.dyn_into::<KeyboardEvent>() {
                state_clone.lock().unwrap().add_event(WindowEvent::KeyDown(
                    js_key_to_key(&event),
                    event.key_code() as i32,
                ));
            }
        }) as Box<dyn FnMut(_)>);
        Self::add_win_event_listener(lstnr, "keydown", cl);
    }
    #[inline]
    fn add_keyup_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            if let Ok(event) = e.dyn_into::<KeyboardEvent>() {
                state_clone.lock().unwrap().add_event(WindowEvent::KeyUp(
                    js_key_to_key(&event),
                    event.key_code() as i32,
                ));
            }
        }) as Box<dyn FnMut(_)>);
        Self::add_win_event_listener(lstnr, "keyup", cl);
    }
    #[inline]
    fn add_keyheld_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            if let Ok(event) = e.dyn_into::<KeyboardEvent>() {
                state_clone.lock().unwrap().add_event(WindowEvent::KeyHeld(
                    js_key_to_key(&event),
                    event.key_code() as i32,
                ));
            }
        }) as Box<dyn FnMut(_)>);
        Self::add_win_event_listener(lstnr, "keyheld", cl);
    }
    #[inline]
    fn add_mousedown_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            if let Ok(event) = e.dyn_into::<MouseEvent>() {
                state_clone
                    .lock()
                    .unwrap()
                    .add_event(WindowEvent::MouseButtonDown(js_mouse_to_mouse(&event)));
            }
        }) as Box<dyn FnMut(_)>);
        self.add_event_listener(lstnr, "mousedown", cl);
    }
    #[inline]
    fn add_mouseup_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            if let Ok(event) = e.dyn_into::<MouseEvent>() {
                state_clone
                    .lock()
                    .unwrap()
                    .add_event(WindowEvent::MouseButtonUp(js_mouse_to_mouse(&event)));
            }
        }) as Box<dyn FnMut(_)>);
        self.add_event_listener(lstnr, "mouseup", cl);
    }
    #[inline]
    fn add_mouseheld_listener(&mut self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            if let Ok(event) = e.dyn_into::<MouseEvent>() {
                state_clone
                    .lock()
                    .unwrap()
                    .add_event(WindowEvent::MouseButtonHeld(js_mouse_to_mouse(&event)));
            }
        }) as Box<dyn FnMut(_)>);
        self.add_event_listener(lstnr, "mouseheld", cl);
    }
    #[inline]
    fn mouse_enter_listener_template(
        &mut self,
        lstnr: &Arc<Mutex<WebEventListener>>,
        name: &'static str,
        enter: bool,
    ) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |_: Event| {
            state_clone
                .lock()
                .unwrap()
                .add_event(WindowEvent::CursorEnter(enter));
        }) as Box<dyn FnMut(_)>);
        self.add_event_listener(lstnr, name, cl);
    }
    #[inline]
    fn focus_listener_template(
        &mut self,
        lstnr: &Arc<Mutex<WebEventListener>>,
        name: &'static str,
        focus: bool,
    ) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |_: Event| {
            state_clone
                .lock()
                .unwrap()
                .add_event(WindowEvent::Focus(focus));
        }) as Box<dyn FnMut(_)>);
        Self::add_win_event_listener(lstnr, name, cl);
    }
}

#[wasm_bindgen(js_name = keyboardEventToKey)]
pub fn js_key_to_key(k: &KeyboardEvent) -> Key {
    let code = k.key_code() as i32;
    match code {
        32 => Key::Space,
        222 => Key::Apostrophe,
        188 => Key::Comma,
        189 => Key::Minus,
        190 => Key::Period,
        191 => Key::Slash,
        48..=57 => Key::from_i32(code),
        186 => Key::Semicolon,
        187 => Key::Equal,
        65..=90 => Key::from_i32(code),
        219 => Key::LeftBracket,
        220 => Key::Backslash,
        221 => Key::RightBracket,
        192 => Key::GraveAccent,
        27 => Key::Escape,
        13 => Key::Enter,
        9 => Key::Tab,
        8 => Key::Backspace,
        45 => Key::Insert,
        46 => Key::Delete,
        39 => Key::Right,
        37 => Key::Left,
        40 => Key::Down,
        38 => Key::Up,
        33 => Key::PageUp,
        34 => Key::PageDown,
        36 => Key::Home,
        35 => Key::End,
        20 => Key::CapsLock,
        145 => Key::ScrollLock,
        144 => Key::NumLock,
        44 => Key::PrintScreen,
        19 => Key::Pause,
        112..=136 => Key::from_i32((code - 112) + 290),
        16 => match k.code().as_str() {
            "ShiftLeft" => Key::LeftShift,
            "ShiftRight" => Key::RightShift,
            _ => Key::Unknown,
        },
        17 => match k.code().as_str() {
            "ControlLeft" => Key::LeftControl,
            "ControlRight" => Key::RightControl,
            _ => Key::Unknown,
        },
        18 => match k.code().as_str() {
            "AltLeft" => Key::LeftAlt,
            "AltRight" => Key::RightAlt,
            _ => Key::Unknown,
        },
        91 => match k.code().as_str() {
            "MetaLeft" => Key::LeftSuper,
            "MetaRight" => Key::RightSuper,
            _ => Key::Unknown,
        },
        93 => Key::Menu,
        _ => Key::Unknown,
    }
}
#[wasm_bindgen(js_name = mouseEventToMouseButton)]
pub fn js_mouse_to_mouse(m: &MouseEvent) -> MouseButton {
    let button = m.button() as i32;
    match button {
        0..=4 => MouseButton::from_i32(button),
        _ => MouseButton::Unknown,
    }
}

#[wasm_bindgen]
pub struct WebGlContext {
    context: Arc<Mutex<WebGl2RenderingContext>>,
    features: HashSet<GlFeature>,
}
impl WebGlContext {
    fn gl_feature(f: &GlFeature) -> u32 {
        use GlFeature::*;
        match f {
            Blend => WebGl2RenderingContext::BLEND,
            CullFace => WebGl2RenderingContext::CULL_FACE,
            DepthTest => WebGl2RenderingContext::DEPTH_TEST,
            Dither => WebGl2RenderingContext::DITHER,
            PolygonOffsetFill => WebGl2RenderingContext::POLYGON_OFFSET_FILL,
            SampleAlphaToCoverage => WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE,
            SampleCoverage => WebGl2RenderingContext::SAMPLE_COVERAGE,
            ScissorTest => WebGl2RenderingContext::SCISSOR_TEST,
            StencilTest => WebGl2RenderingContext::STENCIL_TEST,
            TextureCubeMap => WebGl2RenderingContext::TEXTURE_CUBE_MAP,
            _ => {
                char_panic!("WebGL: GlFeature {:?} not supported on web.", f);
            }
        }
    }
}
impl GlContext for WebGlContext {
    fn new(w: &mut Window) -> Self {
        Self {
            context: w.get_context_arc(),
            features: HashSet::with_capacity(10),
        }
    }

    fn clear(&self, mask: &[GlClearMask]) {
        use GlClearMask::*;
        let mut gl_mask = 0;
        for i in 0..mask.len() {
            gl_mask |= match mask[i] {
                Color => WebGl2RenderingContext::COLOR_BUFFER_BIT,
                Depth => WebGl2RenderingContext::DEPTH_BUFFER_BIT,
                Stencil => WebGl2RenderingContext::STENCIL_BUFFER_BIT,
                _ => 0,
            };
        }
        self.context.lock().unwrap().clear(gl_mask);
    }
    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.context.lock().unwrap().clear_color(r, g, b, a);
    }
    fn viewport(&self, x: i32, y: i32, w: u32, h: u32) {
        self.context
            .lock()
            .unwrap()
            .viewport(x, y, w as i32, h as i32);
    }
    fn enable(&mut self, feature: GlFeature) {
        self.features.insert(feature);
        self.context
            .lock()
            .unwrap()
            .enable(Self::gl_feature(&feature));
    }
    fn disable(&mut self, feature: GlFeature) {
        self.features.remove(&feature);
        self.context
            .lock()
            .unwrap()
            .disable(Self::gl_feature(&feature));
    }
    fn get_enabled_features(&self) -> Vec<GlFeature> {
        self.features.iter().map(|x| *x).collect()
    }
    fn default_depth_func(&self) {
        self.context
            .lock()
            .unwrap()
            .depth_func(WebGl2RenderingContext::LEQUAL);
    }
}

#[wasm_bindgen]
pub struct WebGlBuffer {
    buff: Option<JsSysWebGlBuffer>,
    context: Arc<Mutex<WebGl2RenderingContext>>,
    buff_type: GlBufferType,
    gl_buff: u32,
}
impl WebGlBuffer {
    fn buff_type(t: &GlBufferType) -> u32 {
        use GlBufferType::*;
        match t {
            ArrayBuffer => WebGl2RenderingContext::ARRAY_BUFFER,
            CopyReadBuffer => WebGl2RenderingContext::COPY_READ_BUFFER,
            CopyWriteBuffer => WebGl2RenderingContext::COPY_WRITE_BUFFER,
            ElementArrayBuffer => WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            PixelPackBuffer => WebGl2RenderingContext::PIXEL_PACK_BUFFER,
            PixelUnpackBuffer => WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
            TransformFeedbackBuffer => WebGl2RenderingContext::TRANSFORM_FEEDBACK_BUFFER,
            UniformBuffer => WebGl2RenderingContext::UNIFORM_BUFFER,
            _ => {
                char_panic!("WebGL: Buffer type \"{:?}\" not supported.", t);
            }
        }
    }
    fn storage_mode(t: &GlStorageMode) -> u32 {
        use GlStorageMode::*;
        match t {
            Static => WebGl2RenderingContext::STATIC_DRAW,
            Dynamic => WebGl2RenderingContext::DYNAMIC_DRAW,
        }
    }
}
impl GlBindable for WebGlBuffer {
    fn bind(&self) {
        self.context
            .lock()
            .unwrap()
            .bind_buffer(self.gl_buff, self.buff.as_ref());
    }
    fn unbind(&self) {
        self.context.lock().unwrap().bind_buffer(self.gl_buff, None);
    }
}
impl GlBuffer for WebGlBuffer {
    fn new(w: &Window, tp: GlBufferType) -> Self {
        Self {
            buff: w.get_context_arc().lock().unwrap().create_buffer(),
            context: w.get_context_arc(),
            buff_type: tp,
            gl_buff: Self::buff_type(&tp),
        }
    }
    fn buffer_data(&self, size: usize, data: *const f32, mode: GlStorageMode) {
        unsafe {
            let positions_array_buf_view =
                Float32Array::view_mut_raw(data as *mut f32, size / size_of::<f32>());
            self.context
                .lock()
                .unwrap()
                .buffer_data_with_array_buffer_view(
                    self.gl_buff,
                    &positions_array_buf_view,
                    Self::storage_mode(&mode),
                );
        }
    }
    fn buffer_sub_data(&self, start: usize, size: usize, data: *const f32) {
        unsafe {
            let positions_array_buf_view =
                Float32Array::view_mut_raw(data as *mut f32, size / size_of::<f32>());
            self.context
                .lock()
                .unwrap()
                .buffer_sub_data_with_i32_and_array_buffer_view(
                    self.gl_buff,
                    start as i32,
                    &positions_array_buf_view,
                );
        }
    }
    fn get_buffer_sub_data(&self, start: usize, size: usize, recv: *mut f32) {
        let size = size / size_of::<f32>();
        let start = start / size_of::<f32>();
        unsafe {
            let positions_array_buf_view = Float32Array::view_mut_raw(recv, size);
            self.context
                .lock()
                .unwrap()
                .get_buffer_sub_data_with_i32_and_array_buffer_view(
                    self.gl_buff,
                    start as i32,
                    &positions_array_buf_view,
                );
        }
    }
    fn get_type(&self) -> GlBufferType {
        self.buff_type
    }
}
impl Drop for WebGlBuffer {
    fn drop(&mut self) {
        self.context
            .lock()
            .unwrap()
            .delete_buffer(self.buff.as_ref());
    }
}

#[wasm_bindgen]
pub struct WebGlShader {
    shader: JsSysWebGlShader,
    context: Arc<Mutex<WebGl2RenderingContext>>,
    stype: GlShaderType,
}
impl WebGlShader {
    fn shader_type(t: &GlShaderType) -> u32 {
        use GlShaderType::*;
        match t {
            Vertex => WebGl2RenderingContext::VERTEX_SHADER,
            Fragment => WebGl2RenderingContext::FRAGMENT_SHADER,
            _ => {
                char_panic!("WebGL: shader type \"{:?}\" not supported.", t);
            }
        }
    }
    fn get_shader_ref(&self) -> &JsSysWebGlShader {
        &self.shader
    }
}
impl GlShader for WebGlShader {
    fn new(w: &Window, st: GlShaderType) -> Self {
        let shader_type_u32 = Self::shader_type(&st);
        Self {
            shader: w
                .get_context_arc()
                .lock()
                .unwrap()
                .create_shader(shader_type_u32)
                .unwrap_or_else(|| {
                    char_panic!("WebGL: Could not create shader.");
                }),
            stype: st,
            context: w.get_context_arc(),
        }
    }
    fn shader_source(&self, src: &str) {
        self.context
            .lock()
            .unwrap()
            .shader_source(&self.shader, src);
    }
    fn compile(&self) {
        self.context.lock().unwrap().compile_shader(&self.shader);
    }
    fn get_compile_status(&self) -> Option<String> {
        let gl = self.context.lock().unwrap();
        if !gl
            .get_shader_parameter(&self.shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(true)
        {
            Some(
                format!(
                    "WebGL: Shader compilation error: {}",
                    gl.get_shader_info_log(&self.shader)
                        .unwrap_or_else(|| String::from("Unknown error."))
                )
                .into(),
            )
        } else {
            None
        }
    }
    fn get_type(&self) -> GlShaderType {
        self.stype
    }
}
impl Drop for WebGlShader {
    fn drop(&mut self) {
        self.context
            .lock()
            .unwrap()
            .delete_shader(Some(&self.shader));
    }
}

#[wasm_bindgen]
pub struct WebGlVertexArray {
    vao: Option<JsSysWebGlVertexArray>,
    context: Arc<Mutex<WebGl2RenderingContext>>,
}
impl GlBindable for WebGlVertexArray {
    fn bind(&self) {
        self.context
            .lock()
            .unwrap()
            .bind_vertex_array(self.vao.as_ref());
    }
    fn unbind(&self) {
        self.context.lock().unwrap().bind_vertex_array(None);
    }
}
impl GlVertexArray for WebGlVertexArray {
    fn new(w: &Window) -> Self {
        Self {
            vao: w.get_context_arc().lock().unwrap().create_vertex_array(),
            context: w.get_context_arc(),
        }
    }
    fn attrib_ptr(&self, v: &VertexAttrib) {
        self.context.lock().unwrap().vertex_attrib_pointer_with_i32(
            v.0,
            v.1 as i32,
            WebGl2RenderingContext::FLOAT,
            false,
            v.2 as i32,
            v.3 as i32,
        );
        self.context.lock().unwrap().enable_vertex_attrib_array(v.0);
    }
    fn remove_attrib_ptr(&self, v: &VertexAttrib) {
        self.context
            .lock()
            .unwrap()
            .disable_vertex_attrib_array(v.0);
    }
}
impl Drop for WebGlVertexArray {
    fn drop(&mut self) {
        self.context
            .lock()
            .unwrap()
            .delete_vertex_array(self.vao.as_ref());
    }
}

#[wasm_bindgen]
pub struct WebGlShaderLoc {
    loc: Option<JsSysWebGlUniformLocation>,
}
impl WebGlShaderLoc {
    pub fn loc_ref(&self) -> Option<&JsSysWebGlUniformLocation> {
        self.loc.as_ref()
    }
}
impl GlShaderLoc for WebGlShaderLoc {}

#[wasm_bindgen]
pub struct WebGlProgram {
    program: Option<JsSysWebGlProgram>,
    context: Arc<Mutex<WebGl2RenderingContext>>,
}
impl WebGlProgram {
    fn draw_mode(m: &GlDrawMode) -> u32 {
        use GlDrawMode::*;
        match m {
            Triangles => WebGl2RenderingContext::TRIANGLES,
            Points => WebGl2RenderingContext::POINTS,
            LineStrip => WebGl2RenderingContext::LINE_STRIP,
            LineLoop => WebGl2RenderingContext::LINE_LOOP,
            Lines => WebGl2RenderingContext::LINES,
            TriangleStrip => WebGl2RenderingContext::TRIANGLE_STRIP,
            TriangleFan => WebGl2RenderingContext::TRIANGLE_FAN,
        }
    }
}
impl GlBindable for WebGlProgram {
    fn bind(&self) {
        self.context
            .lock()
            .unwrap()
            .use_program(self.program.as_ref());
    }
    fn unbind(&self) {
        self.context.lock().unwrap().use_program(None);
    }
}
impl GlProgram for WebGlProgram {
    type ShaderLoc = WebGlShaderLoc;
    type Shader = WebGlShader;

    fn new(w: &Window) -> Self {
        Self {
            program: w.get_context_arc().lock().unwrap().create_program(),
            context: w.get_context_arc(),
        }
    }
    fn draw_arrays(&self, mode: GlDrawMode, start: i32, len: i32) {
        self.context
            .lock()
            .unwrap()
            .draw_arrays(Self::draw_mode(&mode), start, len);
    }
    fn shader_loc(&self, name: &str) -> Self::ShaderLoc {
        Self::ShaderLoc {
            loc: self
                .context
                .lock()
                .unwrap()
                .get_uniform_location(self.program.as_ref().unwrap(), name),
        }
    }
    fn attach_shader(&self, shader: &Self::Shader) {
        self.context
            .lock()
            .unwrap()
            .attach_shader(self.program.as_ref().unwrap(), shader.get_shader_ref());
    }
    fn link_program(&self) {
        self.context
            .lock()
            .unwrap()
            .link_program(self.program.as_ref().unwrap());
    }
    fn get_link_status(&self) -> Option<String> {
        let gl = self.context.lock().unwrap();
        if !gl
            .get_program_parameter(
                self.program.as_ref().unwrap(),
                WebGl2RenderingContext::LINK_STATUS,
            )
            .as_bool()
            .unwrap_or(true)
        {
            Some(
                format!(
                    "WebGL: Program compilation error: {}",
                    gl.get_program_info_log(self.program.as_ref().unwrap())
                        .unwrap_or_else(|| String::from("Unknown error."))
                )
                .into(),
            )
        } else {
            None
        }
    }
    fn uniform_4f(&self, loc: &Self::ShaderLoc, v: (f32, f32, f32, f32)) {
        self.context
            .lock()
            .unwrap()
            .uniform4f(loc.loc_ref(), v.0, v.1, v.2, v.3);
    }
    fn uniform_3f(&self, loc: &Self::ShaderLoc, v: (f32, f32, f32)) {
        self.context
            .lock()
            .unwrap()
            .uniform3f(loc.loc_ref(), v.0, v.1, v.2);
    }
    fn uniform_2f(&self, loc: &Self::ShaderLoc, v: (f32, f32)) {
        self.context
            .lock()
            .unwrap()
            .uniform2f(loc.loc_ref(), v.0, v.1);
    }
    fn uniform_1f(&self, loc: &Self::ShaderLoc, v: f32) {
        self.context.lock().unwrap().uniform1f(loc.loc.as_ref(), v);
    }
    fn uniform_4i(&self, loc: &Self::ShaderLoc, v: (i32, i32, i32, i32)) {
        self.context
            .lock()
            .unwrap()
            .uniform4i(loc.loc_ref(), v.0, v.1, v.2, v.3);
    }
    fn uniform_3i(&self, loc: &Self::ShaderLoc, v: (i32, i32, i32)) {
        self.context
            .lock()
            .unwrap()
            .uniform3i(loc.loc_ref(), v.0, v.1, v.2);
    }
    fn uniform_2i(&self, loc: &Self::ShaderLoc, v: (i32, i32)) {
        self.context
            .lock()
            .unwrap()
            .uniform2i(loc.loc_ref(), v.0, v.1);
    }
    fn uniform_1i(&self, loc: &Self::ShaderLoc, v: i32) {
        self.context.lock().unwrap().uniform1i(loc.loc_ref(), v);
    }
    fn uniform_mat4f(&self, loc: &Self::ShaderLoc, v: &[f32]) {
        self.context
            .lock()
            .unwrap()
            .uniform_matrix4fv_with_f32_array(loc.loc_ref(), false, v);
    }
    fn uniform_mat3f(&self, loc: &Self::ShaderLoc, v: &[f32]) {
        self.context
            .lock()
            .unwrap()
            .uniform_matrix3fv_with_f32_array(loc.loc_ref(), false, v);
    }
    fn uniform_mat2f(&self, loc: &Self::ShaderLoc, v: &[f32]) {
        self.context
            .lock()
            .unwrap()
            .uniform_matrix2fv_with_f32_array(loc.loc_ref(), false, v);
    }
}
impl Drop for WebGlProgram {
    fn drop(&mut self) {
        self.context
            .lock()
            .unwrap()
            .delete_program(self.program.as_ref());
    }
}

pub struct WebGlTexture2D {
    tex: Option<JsSysWebGlTexture>,
    context: Arc<Mutex<WebGl2RenderingContext>>,
    slot: u32,
}
impl WebGlTexture2D {
    pub fn gl_texture_type(t: &GlTextureType) -> u32 {
        use GlTextureType::*;
        match t {
            Texture2D => WebGl2RenderingContext::TEXTURE_2D,
            CubeMapPositiveX => WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X,
            CubeMapNegativeX => WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X,
            CubeMapPositiveY => WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y,
            CubeMapNegativeY => WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            CubeMapPositiveZ => WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z,
            CubeMapNegativeZ => WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z,
            _ => {
                char_panic!("WebGL: GlTextureType {:?} not supported on web.", t);
            }
        }
    }
    pub fn gl_internal_fmt(f: &GlInternalTextureFormat) -> u32 {
        use GlInternalTextureFormat::*;
        match f {
            Red => WebGl2RenderingContext::RED,
            RG => WebGl2RenderingContext::RG,
            RGB => WebGl2RenderingContext::RGB,
            RGBA => WebGl2RenderingContext::RGBA,
            R8 => WebGl2RenderingContext::R8,
            R8SNorm => WebGl2RenderingContext::R8_SNORM,
            RG8 => WebGl2RenderingContext::RG8,
            RG8SNorm => WebGl2RenderingContext::RG8_SNORM,
            RGB8 => WebGl2RenderingContext::RGB8,
            RGB8SNorm => WebGl2RenderingContext::RGB8_SNORM,
            RGBA4 => WebGl2RenderingContext::RGBA4,
            RGB5A1 => WebGl2RenderingContext::RGB5_A1,
            RGBA8 => WebGl2RenderingContext::RGBA8,
            RGBA8SNorm => WebGl2RenderingContext::RGBA8_SNORM,
            RGB10A2 => WebGl2RenderingContext::RGB10_A2,
            RGB10A2UI => WebGl2RenderingContext::RGB10_A2UI,
            SRGB8 => WebGl2RenderingContext::SRGB8,
            SRGB8Alpha8 => WebGl2RenderingContext::SRGB8_ALPHA8,
            R16F => WebGl2RenderingContext::R16F,
            RG16F => WebGl2RenderingContext::RG16F,
            RGBA16F => WebGl2RenderingContext::RGBA16F,
            R32F => WebGl2RenderingContext::R32F,
            RG32F => WebGl2RenderingContext::RG32F,
            RGBA32F => WebGl2RenderingContext::RGBA32F,
            R11FG11FB10F => WebGl2RenderingContext::R11F_G11F_B10F,
            RGB9E5 => WebGl2RenderingContext::RGB9_E5,
            R8I => WebGl2RenderingContext::R8I,
            R8UI => WebGl2RenderingContext::R8UI,
            R16I => WebGl2RenderingContext::R16I,
            R16UI => WebGl2RenderingContext::R16UI,
            R32I => WebGl2RenderingContext::R32I,
            R32UI => WebGl2RenderingContext::R32UI,
            RG8I => WebGl2RenderingContext::RG8I,
            RG8UI => WebGl2RenderingContext::RG8UI,
            RG16I => WebGl2RenderingContext::RG16I,
            RG16UI => WebGl2RenderingContext::RG16UI,
            RG32I => WebGl2RenderingContext::RG32I,
            RG32UI => WebGl2RenderingContext::RG32UI,
            RGBA8I => WebGl2RenderingContext::RGBA8I,
            RGBA16I => WebGl2RenderingContext::RGBA16I,
            RGBA32I => WebGl2RenderingContext::RGBA32I,
            _ => {
                char_panic!(
                    "WebGL: GlInternalTextureFormat {:?} not supported on web.",
                    f
                );
            }
        }
    }
    pub fn gl_img_fmt(f: &GlImagePixelFormat) -> u32 {
        use GlImagePixelFormat::*;
        match f {
            Red => WebGl2RenderingContext::RED,
            RG => WebGl2RenderingContext::RG,
            RGB => WebGl2RenderingContext::RGB,
            RGBA => WebGl2RenderingContext::RGBA,
            RedInt => WebGl2RenderingContext::RED_INTEGER,
            RGInt => WebGl2RenderingContext::RG_INTEGER,
            RGBInt => WebGl2RenderingContext::RGB_INTEGER,
            RGBAInt => WebGl2RenderingContext::RGBA_INTEGER,
            DepthComponent => WebGl2RenderingContext::DEPTH_COMPONENT,
            DepthStencil => WebGl2RenderingContext::DEPTH_STENCIL,
            _ => {
                char_panic!("WebGL: GlImagePixelFormat {:?} not supported on web.", f);
            }
        }
    }
    pub fn gl_px_fmt(f: &GlImagePixelType) -> u32 {
        use GlImagePixelType::*;
        match f {
            UnsignedByte => WebGl2RenderingContext::UNSIGNED_BYTE,
            Byte => WebGl2RenderingContext::BYTE,
            UnsignedShort => WebGl2RenderingContext::UNSIGNED_SHORT,
            Short => WebGl2RenderingContext::SHORT,
            UnsignedInt => WebGl2RenderingContext::UNSIGNED_INT,
            Int => WebGl2RenderingContext::INT,
            HalfFloat => WebGl2RenderingContext::HALF_FLOAT,
            Float => WebGl2RenderingContext::FLOAT,
        }
    }
    pub fn set_params(&self, mips: Option<u32>) {
        let gl = self.context.lock().unwrap();
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::REPEAT as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::REPEAT as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        if mips == Some(0) {
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
        } else {
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
        }
    }
}
impl GlBindable for WebGlTexture2D {
    fn bind(&self) {
        self.context.lock().unwrap().active_texture(self.slot);
        self.context
            .lock()
            .unwrap()
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, self.tex.as_ref());
    }
    fn unbind(&self) {
        self.context.lock().unwrap().active_texture(self.slot);
        self.context
            .lock()
            .unwrap()
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }
}
impl GlTexture2D for WebGlTexture2D {
    fn new(w: &mut Window) -> Self {
        Self {
            context: w.get_context_arc(),
            tex: w.get_context_arc().lock().unwrap().create_texture(),
            slot: WebGl2RenderingContext::TEXTURE0,
        }
    }
    fn set_texture(
        &self,
        tex_ptr: *const u8,
        width: u32,
        height: u32,
        internal_fmt: GlInternalTextureFormat,
        img_fmt: GlImagePixelFormat,
        px_type: GlImagePixelType,
        mipmaps: Option<u32>,
        px_byte_size: usize,
    ) {
        unsafe {
            let array_buff_view = Uint8Array::view_mut_raw(
                tex_ptr as *mut u8,
                (width * height) as usize * px_byte_size,
            );
            let mips = if u32::is_power_of_two(width) && u32::is_power_of_two(height) {
                mipmaps
            } else {
                if mipmaps != None {
                    js_warn_string(&format!(
                        "Image improper size ({}x{}) to support specific mipmap level {}.",
                        width,
                        height,
                        mipmaps.unwrap()
                    ));
                }
                None
            };
            self.context.lock().unwrap()
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
                    WebGl2RenderingContext::TEXTURE_2D,
                    mips.unwrap_or_else(|| 0) as i32,
                    Self::gl_internal_fmt(&internal_fmt) as i32,
                    width as i32,
                    height as i32,
                    0,
                    Self::gl_img_fmt(&img_fmt),
                    Self::gl_px_fmt(&px_type),
					&array_buff_view,
					0,
                )
                .unwrap_or_else(|err| {
                    char_panic!("WebGL: Error calling texImage2D: {:?}.", err);
                });
            self.set_params(mips);
        }
    }
    fn set_slot(&mut self, slot: u32) {
        self.slot = WebGl2RenderingContext::TEXTURE0 + slot;
    }
}
impl Drop for WebGlTexture2D {
    fn drop(&mut self) {
        self.context
            .lock()
            .unwrap()
            .delete_texture(self.tex.as_ref());
    }
}
