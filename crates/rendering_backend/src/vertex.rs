use nalgebra::{Vector2, Vector3};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub color: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
    pub normal: Vector3<f32>,
    pub texture_index: u32,
}
impl Default for Vertex {
    fn default() -> Vertex {
        Vertex {
            pos: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            tex_coord: Vector2::new(0.0, 0.0),
            color: Vector3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        }
    }
}
