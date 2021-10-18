use crate::platform::Window;
use crate::window::EventManager;

pub trait State: 'static {
    fn initialize(&mut self, win: &mut Window, manager: &mut dyn EventManager) -> i32;
    fn update(&mut self, win: &mut Window, manager: &mut dyn EventManager, delta: f64) -> i32;
    fn destroy(&mut self, win: &mut Window, manager: &mut dyn EventManager, exit_code: i32);
}

#[cfg(not(target_family = "wasm"))]
#[derive(Debug)]
pub struct FrameManager {
    delta: f64,
    target_delta_micro: Option<u128>,
    last_frame_micro: u128,
}
#[cfg(not(target_family = "wasm"))]
impl FrameManager {
    pub fn new(fps: Option<f64>) -> FrameManager {
        let mut ret = FrameManager {
            delta: 0f64,
            target_delta_micro: None,
            last_frame_micro: Self::current_time_micro(),
        };
        ret.set_fps(fps);
        ret
    }
    pub fn current_time_micro() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("FrameManager: Time went backwards.")
            .as_micros() as u128
    }
    pub fn next_frame_ready(&mut self) -> bool {
		if let Some(target_delta_mcs) = self.target_delta_micro {
			if Self::current_time_micro() - self.last_frame_micro >= target_delta_mcs {
				self.delta = (Self::current_time_micro() - self.last_frame_micro) as f64 / 1000000f64;
				self.last_frame_micro = Self::current_time_micro();
				true
			} else {
				false
			}
		} else {
			self.delta = (Self::current_time_micro() - self.last_frame_micro) as f64 / 1000000f64;
			self.last_frame_micro = Self::current_time_micro();
			true
		}
    }
    pub fn set_fps(&mut self, fps: Option<f64>) {
		if let Some(fps) = fps {
        	self.target_delta_micro = Some(((1f64 / fps) * 1000f64) as u128 * 1000u128);
		} else {
			self.target_delta_micro = None;
		}
    }
    pub fn get_delta(&self) -> f64 {
        self.delta
    }
}
