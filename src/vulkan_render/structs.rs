use std::mem::offset_of;

use ash::vk;
use cgmath::{Matrix4, Vector2, Vector3};
use serde::Serialize;

pub struct FrameData {
    pub render_semaphore: vk::Semaphore,
    pub swapchain_semaphore: vk::Semaphore,
    pub render_fence: vk::Fence,
    pub command_buffer: vk::CommandBuffer,
}

pub struct GPUMeshData {
    pub vertex_buffer: AllocatedBuffer,
    pub index_buffer: AllocatedBuffer,
    pub index_count: u32,
    //pub vertex_buffer_address: vk::DeviceAddress,
}

#[derive(Serialize)]
pub struct PushConstants {
    pub vertex_buffer_address: vk::DeviceAddress,
}

pub struct AllocatedBuffer {
    pub buffer: vk::Buffer,
    pub buffer_memory: vk::DeviceMemory,
}

pub struct AllocatedImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_extent: vk::Extent3D,
    pub image_format: vk::Format,
    pub image_memory: vk::DeviceMemory,
}

pub struct Material {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub color: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

impl Default for Vertex {
    fn default() -> Vertex {
        Vertex {
            pos: Vector3::new(0.0,0.0,0.0),
            color: Vector3::new(0.0,0.0,0.0),
            tex_coord: Vector2::new(0.0, 0.0,),
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

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, tex_coord) as u32,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}
