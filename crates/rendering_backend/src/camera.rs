use nalgebra::Matrix4;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct CameraMvpUbo {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}