
pub trait CPUBuffer {
	fn to_gpu_buffer(&self) -> Self;
}
pub trait GPUBuffer {
	fn to_cpu_buffer(&self) -> Self;
}

pub struct CPUTexture {}
impl CPUBuffer for CPUTexture {}

pub struct GPUTexture {}
impl GPUBuffer for GPUTexture {}

pub struct GPUTriArray {}
impl GPUBuffer for GPUTriArray {}

pub struct CPUTriArray {}
impl CPUBuffer for CPUTriArray {}