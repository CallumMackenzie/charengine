pub mod c3d;

use crate::platform::Window;

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
    fn new() -> Self;
    fn set_data(&mut self, win: &mut Window, data: &Self::Data);
    fn get_data(&self, win: &mut Window) -> Self::Data;

    fn from_data(win: &mut Window, data: &Self::Data) -> Self {
        let mut ret = Self::new();
        ret.set_data(win, data);
        ret
    }
    fn to_cpu_buffer(&self, win: &mut Window) -> Self::CPUType {
        Self::CPUType::from_data(&self.get_data(win))
    }
}

pub trait GPUShader {
    fn new() -> Self;
    fn compile(&self, vertex: &str, fragment: &str);
    fn use_shader(&self);
}
