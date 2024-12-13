use cgmath::{Vector2, Vector3};
use vulkan_render::structs::Vertex;
pub mod vulkan_render;
pub const VERTICES: [Vertex; 4] = [
    Vertex {
        pos: Vector3::new(-1.0, 0.0, 1.0),
        color: Vector3::new(1.0, 0.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
    },
    Vertex {
        pos: Vector3::new(1.0, 0.0, 1.0),
        color: Vector3::new(0.0, 1.0, 0.0),
        tex_coord: Vector2::new(1.0, 0.0),
    },
    Vertex {
        pos: Vector3::new(1.0, 0.0, -1.0),
        color: Vector3::new(0.0, 0.0, 1.0),
        tex_coord: Vector2::new(1.0, 1.0),
    },
    Vertex {
        pos: Vector3::new(-1.0, 0.0, -1.0),
        color: Vector3::new(1.0, 1.0, 1.0),
        tex_coord: Vector2::new(0.0, 1.0),
    }
];

pub const INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];
