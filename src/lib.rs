use vulkan_render::structs::Vertex;

pub mod vulkan_render;
pub const VERTICES: [Vertex; 6] = [
    Vertex {
        pos: [0.0, -0.75],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [-0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.0, 0.75],
        color: [1.0, 1.0, 1.0],
    },
];
