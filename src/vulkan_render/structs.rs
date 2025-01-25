use std::mem::offset_of;

use crate::vulkan_render::buffer::AllocatedBuffer;
use crate::vulkan_render::device::DeviceInfo;
use crate::vulkan_render::image_util;
use crate::vulkan_render::image_util::AllocatedImage;
use ash::vk::{
    Extent3D, Format, ImageAspectFlags, ImageSubresourceRange, ImageTiling, ImageUsageFlags,
    ImageViewCreateInfo, MemoryPropertyFlags, Sampler, SamplerAddressMode, SamplerCreateInfo,
    SamplerMipmapMode,
};
use ash::{vk, Instance};
use glm::{Vec4};
use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use serde::Serialize;

pub struct GPUMeshData {
    pub vertex_buffer: AllocatedBuffer,
    pub index_buffer: AllocatedBuffer,
    pub index_count: u32,
    pub world_model: Matrix4<f32>,
}

#[derive(Serialize)]
pub struct PushConstants {
    pub vertex_buffer_address: vk::DeviceAddress,
}

/*pub struct AllocatedBuffer {
    pub buffer: vk::Buffer,
    pub buffer_memory: DeviceMemory,
    pub mapped_buffer: *mut c_void,
}*/

pub struct Material {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

pub struct ShadowMap {
    pub shadow_image: AllocatedImage,
    pub sampler: Sampler,
}

impl ShadowMap {
    pub fn new(instance: &Instance, device_info: &DeviceInfo) -> Self {
        let shadow_extend = Extent3D {
            width: 1024,
            height: 1024,
            depth: 1,
        };

        let (image, image_memory) = image_util::create_image(
            device_info,
            instance,
            shadow_extend.width,
            shadow_extend.height,
            Format::D16_UNORM,
            ImageTiling::OPTIMAL,
            ImageUsageFlags::SAMPLED | ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let image_view_create_info = ImageViewCreateInfo::default()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(Format::D16_UNORM)
            .subresource_range(ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image(image);

        let image_view = unsafe {
            device_info
                .logical_device
                .create_image_view(&image_view_create_info, None)
                .expect("failed to create shadow image view")
        };

        let sampler_create_info = SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(SamplerMipmapMode::LINEAR)
            .address_mode_u(SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(SamplerAddressMode::CLAMP_TO_EDGE)
            .mip_lod_bias(0.0)
            .max_anisotropy(16.0)
            .min_lod(0.0)
            .max_lod(1.0)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE);

        let sampler = unsafe {
            device_info
                .logical_device
                .create_sampler(&sampler_create_info, None)
                .expect("failed to create shadow sampler")
        };

        let shadow_image = AllocatedImage {
            image,
            image_view,
            image_memory,
            image_format: Format::D16_UNORM,
            image_extent: shadow_extend,
        };

        Self {
            shadow_image,
            sampler,
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub color: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
    pub normal: Vector3<f32>,
}

impl Default for Vertex {
    fn default() -> Vertex {
        Vertex {
            pos: Vector3::new(0.0, 0.0, 0.0),
            color: Vector3::new(0.0, 0.0, 0.0),
            tex_coord: Vector2::new(0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
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

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct CameraMvpUbo {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ModelDynamicUbo {
    pub model: Matrix4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct LightingUbo {
    pub light_direction: Vector4<f32>,
    pub light_color: Vector4<f32>,
    pub ambient_light: Vector4<f32>,
    pub cascade_depths: Vector4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct CascadeShadowUbo {
    pub cascade_view_proj: Matrix4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct Cascade {
    pub cascade_view_proj: Matrix4<f32>,
    pub cascade_depth: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy, Serialize)]
pub struct CascadeShadowPushConsts {
    pub pos: [f32; 4],
    pub index: u32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct LineVertex {
   pub pos: Vec4,
}
