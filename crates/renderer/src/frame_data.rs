use nalgebra_glm::Mat4;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::buffer::{BufferDesc, BufferHandle, BufferUsageFlags};
use rendering_backend::camera::CameraMvpUbo;
use rendering_backend::descriptor::{
    DescriptorBinding, DescriptorLayoutDesc, DescriptorLayoutHandle, DescriptorSetHandle,
    DescriptorType, DescriptorValue, DescriptorWriteDesc, ShaderStage,
};
use rendering_backend::image::{
    GpuImageHandle, ImageAspect, ImageDesc, ImageUsageFlags, TextureFormat,
};
use rendering_backend::memory::MemoryHint;
use rendering_backend::sampler::{Filter, SamplerAddressMode, SamplerDesc, SamplerHandle};

#[derive(Debug, Clone)]
pub struct FrameImages {
    pub gbuffer_albedo: GpuImageHandle,
    pub gbuffer_normal: GpuImageHandle,
    pub gbuffer_depth: GpuImageHandle,
    pub draw_image: GpuImageHandle,
    pub shadow_cascades: Vec<GpuImageHandle>,
}

impl FrameImages {
    pub fn new(
        vulkan_backend: &mut VulkanBackend,
        resolution_settings: ResolutionSettings,
    ) -> Self {
        let window_resolution = resolution_settings.window_resolution;
        let gbuffer_albedo = vulkan_backend.create_image(ImageDesc {
            width: window_resolution.width,
            height: window_resolution.height,
            depth: 1,
            format: TextureFormat::R16g16b16a16Float,
            clear_value: None,
            array_layers: 1,
            is_cubemap: false,
            mip_levels: 1,
            aspect: ImageAspect::Color,
            usage: ImageUsageFlags::COLOR_ATTACHMENT
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST
                | ImageUsageFlags::SAMPLED
                | ImageUsageFlags::STORAGE,
        });
        let gbuffer_normal = vulkan_backend.create_image(ImageDesc {
            width: window_resolution.width,
            height: window_resolution.height,
            depth: 1,
            format: TextureFormat::R16g16b16a16Float,
            clear_value: None,
            array_layers: 1,
            is_cubemap: false,
            mip_levels: 1,
            aspect: ImageAspect::Color,
            usage: ImageUsageFlags::COLOR_ATTACHMENT
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST
                | ImageUsageFlags::SAMPLED
                | ImageUsageFlags::STORAGE,
        });

        let gbuffer_depth = vulkan_backend.create_image(ImageDesc {
            width: window_resolution.width,
            height: window_resolution.height,
            depth: 1,
            format: TextureFormat::D32Float,
            clear_value: None,
            array_layers: 1,
            is_cubemap: false,
            mip_levels: 1,
            aspect: ImageAspect::Depth,
            usage: ImageUsageFlags::DEPTH_ATTACHMENT
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST
                | ImageUsageFlags::SAMPLED
                | ImageUsageFlags::STORAGE,
        });

        let draw_image = vulkan_backend.create_image(ImageDesc {
            width: window_resolution.width,
            height: window_resolution.height,
            depth: 1,
            format: TextureFormat::R16g16b16a16Float,
            clear_value: None,
            array_layers: 1,
            is_cubemap: false,
            mip_levels: 1,
            aspect: ImageAspect::Color,
            usage: ImageUsageFlags::COLOR_ATTACHMENT
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST
                | ImageUsageFlags::SAMPLED
                | ImageUsageFlags::STORAGE,
        });

        let shadow_cascades = resolution_settings
            .shadow_resolutions
            .iter()
            .map(|res| {
                vulkan_backend.create_image(ImageDesc {
                    width: res.width,
                    height: res.height,
                    depth: 1,
                    format: TextureFormat::D32Float,
                    clear_value: None,
                    array_layers: 1,
                    is_cubemap: false,
                    mip_levels: 1,
                    aspect: ImageAspect::Depth,
                    usage: ImageUsageFlags::DEPTH_ATTACHMENT
                        | ImageUsageFlags::TRANSFER_SRC
                        | ImageUsageFlags::TRANSFER_DST
                        | ImageUsageFlags::SAMPLED
                        | ImageUsageFlags::STORAGE,
                })
            })
            .collect();

        Self {
            gbuffer_albedo,
            gbuffer_normal,
            gbuffer_depth,
            draw_image,
            shadow_cascades,
        }
    }
}

/// Per-frame GPU resources shared across the geometry and debug passes:
/// camera/model data buffers, the frame-level descriptor set, and the basic sampler.
/// Shadow and lighting resources live in LightingRenderer.
pub struct FrameData {
    pub frame_images: FrameImages,
    pub camera_buffer: BufferHandle,
    pub model_storage_buffer: BufferHandle,
    pub descriptor_layout_handle: DescriptorLayoutHandle,
    pub descriptor_handle: DescriptorSetHandle,
    pub basic_sampler: SamplerHandle,
}

impl FrameData {
    pub fn new(
        vulkan_backend: &mut VulkanBackend,
        resolution_settings: ResolutionSettings,
        max_meshes: usize,
    ) -> Self {
        let frame_images = FrameImages::new(vulkan_backend, resolution_settings);
        let buffer_size = size_of::<CameraMvpUbo>();

        let camera_buffer = vulkan_backend.create_buffer::<CameraMvpUbo>(
            BufferDesc {
                size: buffer_size,
                usage: BufferUsageFlags::UNIFORM,
                memory_hint: MemoryHint::CPUWritable,
            },
            None,
        );

        let model_storage_buffer = vulkan_backend.create_buffer::<Mat4>(
            BufferDesc {
                size: size_of::<Mat4>() * max_meshes,
                memory_hint: MemoryHint::CPUWritable,
                usage: BufferUsageFlags::STORAGE,
            },
            None,
        );

        let basic_sampler = vulkan_backend.create_sampler(SamplerDesc {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            address_u: SamplerAddressMode::Repeat,
            address_v: SamplerAddressMode::Repeat,
            address_w: SamplerAddressMode::Repeat,
            compare_enable: false,
            compare_op: None,
        });

        let frame_layout_desc = DescriptorLayoutDesc {
            bindings: vec![
                DescriptorBinding {
                    binding: 0,
                    descriptor_type: DescriptorType::UniformBuffer,
                    count: 1,
                    stages: ShaderStage::VERTEX,
                },
                DescriptorBinding {
                    binding: 1,
                    descriptor_type: DescriptorType::StorageBuffer,
                    count: 1,
                    stages: ShaderStage::VERTEX,
                },
            ],
        };

        let descriptor_layout_handle = vulkan_backend.create_descriptor_layout(frame_layout_desc);
        let descriptor_handle = vulkan_backend.allocate_descriptor_set(descriptor_layout_handle);

        vulkan_backend.update_descriptor_set(
            descriptor_handle,
            &[
                DescriptorWriteDesc {
                    binding: 0,
                    value: DescriptorValue::UniformBuffer(camera_buffer),
                },
                DescriptorWriteDesc {
                    binding: 1,
                    value: DescriptorValue::StorageBuffer(model_storage_buffer),
                },
            ],
        );

        Self {
            frame_images,
            camera_buffer,
            model_storage_buffer,
            descriptor_layout_handle,
            descriptor_handle,
            basic_sampler,
        }
    }
}

pub struct ResolutionSettings {
    pub window_resolution: Resolution,
    pub shadow_resolutions: Vec<Resolution>,
}

pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn get_aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}
