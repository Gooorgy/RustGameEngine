use ash::vk;
use ash::vk::Format;
use nalgebra::{Vector2, Vector3};
use std::mem::offset_of;
use crate::device::DeviceInfo;
use crate::render_objects::render_batch::Draw;

pub enum Drawable {
    Mesh(Mesh),

    // TODO: Implement these?
    FullScreenQuad,
    Particle,
    Terrain,
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Draw for Mesh {
    fn draw(&self, command_buffer: vk::CommandBuffer, device_info: &DeviceInfo) {
        todo!()
    }
}

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

impl Vertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: Format::R32G32_SFLOAT,
                offset: offset_of!(Self, tex_coord) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 3,
                format: Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, normal) as u32,
            },
        ]
    }
}
