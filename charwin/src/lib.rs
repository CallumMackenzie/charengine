pub mod input;
pub mod window;

#[cfg(target_family = "wasm")]
use wasm_bindgen::closure::Closure;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
mod wasm_utils {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    pub fn window() -> web_sys::Window {
        web_sys::window().expect("no global `window` exists")
    }
    pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        window()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }
    pub fn document() -> web_sys::Document {
        window()
            .document()
            .expect("should have a document on window")
    }
    pub fn body() -> web_sys::HtmlElement {
        document().body().expect("document should have a body")
    }
}

#[derive(Debug)]
pub enum AppState {
    Ok,
    Crash(String),
    Exit,
}

pub trait AppLogic {
    fn on_start(&mut self) -> AppState;
    fn on_update(&mut self) -> AppState;
    fn on_close(&mut self) -> AppState;
}

#[cfg(not(target_family = "wasm"))]
pub fn start(logic: &mut dyn AppLogic) -> AppState {
    let mut fm = FrameManager::new(60f64);
    match logic.on_start() {
        AppState::Ok => {
            loop {
                if fm.next_frame_ready() {
                    match logic.on_update() {
                        AppState::Crash(msg) => return AppState::Crash(msg),
                        AppState::Exit => break,
                        AppState::Ok => {}
                    }
                }
            }
            logic.on_close();
            AppState::Exit
        }
        AppState::Crash(msg) => AppState::Crash(msg),
        AppState::Exit => AppState::Exit,
    }
}
#[cfg(target_family = "wasm")]
pub fn start(logic: &'static mut dyn AppLogic) -> AppState {
    match logic.on_start() {
        AppState::Ok => {}
        other => return other,
    }
    let f = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        match logic.on_update() {
            AppState::Ok => {}
            AppState::Exit => {
                logic.on_close();
                let _ = f.borrow_mut().take();
                return;
            }
            AppState::Crash(_) => {
                let _ = f.borrow_mut().take();
                return;
            }
        }
        wasm_utils::request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    wasm_utils::request_animation_frame(g.borrow().as_ref().unwrap());
    AppState::Exit
}

#[cfg(not(target_family = "wasm"))]
#[derive(Debug)]
pub struct FrameManager {
    delta: f64,
    target_delta_micro: u128,
    last_frame_micro: u128,
}
#[cfg(not(target_family = "wasm"))]
impl FrameManager {
    pub fn new(fps: f64) -> FrameManager {
        let mut ret = FrameManager {
            delta: 0f64,
            target_delta_micro: 500u128,
            last_frame_micro: 0u128,
        };
        ret.set_fps(fps);
        ret
    }
    pub fn current_time_micro() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards.")
            .as_micros() as u128
    }
    pub fn next_frame_ready(&mut self) -> bool {
        if Self::current_time_micro() - self.last_frame_micro >= self.target_delta_micro {
            self.delta = (Self::current_time_micro() - self.last_frame_micro) as f64 / 1000f64;
            self.last_frame_micro = Self::current_time_micro();
            true
        } else {
            false
        }
    }
    pub fn set_fps(&mut self, fps: f64) {
        self.target_delta_micro = ((1f64 / fps) * 1000f64) as u128 * 1000u128;
    }
    pub fn get_delta(&self) -> f64 {
        self.delta
    }
}
