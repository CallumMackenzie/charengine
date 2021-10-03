use crate::input::{Key, MouseButton};
use crate::state::State;
use crate::window::{
    AbstractWindow, AbstractWindowFactory, EventManager, WindowCreateArgs, WindowEvent,
};
use js_sys::Function;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Event, FocusEvent, HtmlCanvasElement, KeyboardEvent, MouseEvent, WebGl2RenderingContext,
    WheelEvent,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log_string(a: &str);
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn js_warn_string(a: &str);
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

trait WebEventState<E: Copy + Clone>: 'static {
    fn add_event(&mut self, state: E);
    fn clear(&mut self);
    fn get_events(&mut self) -> Vec<E>;
    fn add_events(&mut self, events: Vec<E>) {
        for i in events {
            self.add_event(i);
        }
    }
}
struct WebEventListener {
    events: Vec<WindowEvent>,
}
impl WebEventState<WindowEvent> for WebEventListener {
    fn add_event(&mut self, state: WindowEvent) {
        self.events.push(state);
    }
    fn clear(&mut self) {
        self.events.clear();
    }
    fn get_events(&mut self) -> Vec<WindowEvent> {
        let mut ret = Vec::<WindowEvent>::new();
        for i in 0..self.events.len() {
            ret.push(self.events[i]);
        }
        ret
    }
}

#[wasm_bindgen]
pub struct WebGlWindow {
    context: WebGl2RenderingContext,
    canvas: Arc<Mutex<HtmlCanvasElement>>,
    clear_mask: u32,
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
    pub fn wset_clear_colour(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.set_clear_colour(r, g, b, a);
    }
    #[wasm_bindgen(js_name = clear)]
    pub fn wclear(&mut self) {
        self.clear();
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
}

impl AbstractWindow for WebGlWindow {
    fn set_fullscreen(&mut self) {
        // Does nothing on WASM
    }
    fn set_windowed(&mut self) {
        // Does nothing on WASM
    }
    fn set_title(&mut self, _title: &str) {
        // Does nothing on WASM
    }
    fn set_size(&mut self, _sz: (i32, i32)) {
        // Does nothing on WASM
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
    fn set_clear_colour(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.context
            .clear_color(r as f32, g as f32, b as f32, a as f32);
    }
    fn clear(&mut self) {
        self.context.clear(self.clear_mask)
    }
    fn get_size(&self) -> (i32, i32) {
        let bounding_rect = self.canvas.lock().unwrap().get_bounding_client_rect();
        (bounding_rect.width() as i32, bounding_rect.height() as i32)
    }
    fn get_pos(&self) -> (i32, i32) {
        let bounding_rect = self.canvas.lock().unwrap().get_bounding_client_rect();
        (bounding_rect.x() as i32, bounding_rect.y() as i32)
    }
}

impl AbstractWindowFactory for WebGlWindow {
    fn create(args: &WindowCreateArgs) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(&args.title).unwrap();
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let mut window = WebGlWindow {
            context: canvas
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap(),
            canvas: Arc::new(Mutex::new(canvas)),
            clear_mask: WebGl2RenderingContext::COLOR_BUFFER_BIT,
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
    }
}

impl WebGlWindow {
    fn add_event_listeners(&mut self) {
        let listener = Arc::new(Mutex::new(WebEventListener { events: Vec::new() }));
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
    fn add_event_listener(&self, name: &str, cl: &Function) {
        self.canvas
            .lock()
            .unwrap()
            .add_event_listener_with_callback(name, cl)
            .unwrap();
    }
    #[inline]
    fn add_win_event_listener(&self, name: &str, cl: &Function) {
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback(name, cl)
            .unwrap();
    }
    #[inline]
    fn add_scroll_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            if let Ok(event) = e.dyn_into::<WheelEvent>() {
                state_clone
                    .lock()
                    .unwrap()
                    .add_event(WindowEvent::Scroll(event.delta_x(), event.delta_y()));
            }
        }) as Box<dyn FnMut(_)>);
        self.add_win_event_listener("wheel", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn add_resize_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let canvas = Arc::clone(&self.canvas);
        let cl = Closure::wrap(Box::new(move |_: Event| {
            let canvas_rect = canvas.lock().unwrap().get_bounding_client_rect();
            state_clone.lock().unwrap().add_event(WindowEvent::Size(
                canvas_rect.width() as i32,
                canvas_rect.height() as i32,
            ));
        }) as Box<dyn FnMut(_)>);
        self.add_win_event_listener("resize", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn add_onclose_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |_: Event| {
            state_clone.lock().unwrap().add_event(WindowEvent::Close);
        }) as Box<dyn FnMut(_)>);
        self.add_win_event_listener("beforeunload", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn add_mouseleave_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
        self.mouse_enter_listener_template(lstnr, "mouseleave", false);
    }
    #[inline]
    fn add_mouseenter_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
        self.mouse_enter_listener_template(lstnr, "mouseenter", true);
    }
    #[inline]
    fn add_focus_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
        self.focus_listener_template(lstnr, "focus", true);
    }
    #[inline]
    fn add_blur_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
        self.focus_listener_template(lstnr, "blur", false);
    }
    #[inline]
    fn add_mousemove_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
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
        self.add_event_listener("mousemove", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn add_keydown_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
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
        self.add_win_event_listener("keydown", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn add_keyup_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
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
        self.add_win_event_listener("keyup", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn add_keyheld_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
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
        self.add_win_event_listener("keyheld", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn add_mousedown_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
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
        self.add_event_listener("mousedown", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn add_mouseup_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
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
        self.add_event_listener("mouseup", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn add_mouseheld_listener(&self, lstnr: &Arc<Mutex<WebEventListener>>) {
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
        self.add_event_listener("mouseheld", cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn mouse_enter_listener_template(
        &self,
        lstnr: &Arc<Mutex<WebEventListener>>,
        name: &str,
        enter: bool,
    ) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |_: Event| {
            state_clone
                .lock()
                .unwrap()
                .add_event(WindowEvent::CursorEnter(enter));
        }) as Box<dyn FnMut(_)>);
        self.add_event_listener(name, cl.as_ref().unchecked_ref());
        cl.forget();
    }
    #[inline]
    fn focus_listener_template(
        &self,
        lstnr: &Arc<Mutex<WebEventListener>>,
        name: &str,
        focus: bool,
    ) {
        let state_clone = Arc::clone(lstnr);
        let cl = Closure::wrap(Box::new(move |e: Event| {
            if let Ok(_) = e.dyn_into::<FocusEvent>() {
                state_clone
                    .lock()
                    .unwrap()
                    .add_event(WindowEvent::Focus(focus));
            }
        }) as Box<dyn FnMut(_)>);
        self.add_win_event_listener(name, cl.as_ref().unchecked_ref());
        cl.forget();
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
                    js_log_string("State initialized.");
                    state_initialized = true;
                } else {
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
                    js_log_string("State destroyed.");
                    let _ = f.borrow_mut().take();
                }
            }) as Box<dyn FnMut()>));
            web_sys::window()
                .unwrap()
                .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .unwrap();
        }
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
