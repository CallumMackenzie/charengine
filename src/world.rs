use charwin::data::{DataBuffer, GPUBuffer, TriGPUBuffer, Triangle, VertexBase};
use charwin::platform::Window;

pub trait MeshBase<V: VertexBase>: Sized {
    fn new(win: &mut Window) -> Self;
    fn n_tris(&self) -> i32;
    fn set_data(&mut self, data: &Vec<Triangle<V>>);
    fn tris_from_obj_data(data: &str) -> Vec<Triangle<V>>;

    fn from_obj_data(win: &mut Window, data: &str) -> Self {
        Self::from_data(win, &Self::tris_from_obj_data(data))
    }
    fn from_data(win: &mut Window, data: &Vec<Triangle<V>>) -> Self {
        let mut ret = Self::new(win);
        ret.set_data(data);
        ret
    }
}

pub struct Mesh3D<V: VertexBase> {
    buffer: TriGPUBuffer<V>,
}
impl<V: VertexBase> MeshBase<V> for Mesh3D<V> {
    fn new(win: &mut Window) -> Self {
        Self {
            buffer: TriGPUBuffer::<V>::new(win),
        }
    }
    fn n_tris(&self) -> i32 {
        self.buffer.n_tris()
    }
    fn set_data(&mut self, data: &Vec<Triangle<V>>) {
        self.buffer.set_data(data);
    }
    fn tris_from_obj_data(_data: &str) -> Vec<Triangle<V>> {
        unimplemented!("tris_from_obj_data");
    }
}
