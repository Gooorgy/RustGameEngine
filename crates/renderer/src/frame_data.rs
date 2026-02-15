use nalgebra_glm::{Mat4, Vec4};
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
use rendering_backend::pipeline::CompareOp;
use rendering_backend::sampler::{Filter, SamplerAddressMode, SamplerDesc, SamplerHandle};
use std::mem;

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
        let shadow_map_resolution = resolution_settings.shadow_resolution;
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

        let shadow_cascade_image_desc = ImageDesc {
            width: shadow_map_resolution.width,
            height: shadow_map_resolution.height,
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
        };

        let shadow_image1 = vulkan_backend.create_image(shadow_cascade_image_desc);
        let shadow_image2 = vulkan_backend.create_image(shadow_cascade_image_desc);
        let shadow_image3 = vulkan_backend.create_image(shadow_cascade_image_desc);
        let shadow_image4 = vulkan_backend.create_image(shadow_cascade_image_desc);

        Self {
            gbuffer_albedo,
            gbuffer_normal,
            gbuffer_depth,
            draw_image,
            shadow_cascades: vec![shadow_image1, shadow_image2, shadow_image3, shadow_image4],
        }
    }
}

pub struct FrameData {
    pub frame_images: FrameImages,
    pub camera_buffer: BufferHandle,
    pub model_storage_buffer: BufferHandle,
    pub descriptor_layout_handle: DescriptorLayoutHandle,
    pub descriptor_handle: DescriptorSetHandle,
    pub basic_sampler: SamplerHandle,
    pub cascade_buffer: BufferHandle,
    pub lighting_buffer: BufferHandle,
    pub shadow_sampler: SamplerHandle,
    pub shadow_descriptor_layout: DescriptorLayoutHandle,
    pub shadow_descriptor_set: DescriptorSetHandle,
    pub lighting_descriptor_layout: DescriptorLayoutHandle,
    pub lighting_descriptor_set: DescriptorSetHandle,
}

impl FrameData {
    pub fn new(
        vulkan_backend: &mut VulkanBackend,
        resolution_settings: ResolutionSettings,
        max_meshes: usize,
    ) -> Self {
        let frame_images = FrameImages::new(vulkan_backend, resolution_settings);
        let buffer_size = mem::size_of::<CameraMvpUbo>();

        let camera_buffer = vulkan_backend.create_buffer::<CameraMvpUbo>(
            BufferDesc {
                size: buffer_size,
                usage: BufferUsageFlags::UNIFORM,
                memory_hint: MemoryHint::CPUWritable,
            },
            None,
        );

        let model_storage_buffer = vulkan_backend.create_buffer::<CameraMvpUbo>(
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

        // Shadow/lighting resources
        const CASCADE_COUNT: usize = 3;

        let cascade_buffer = vulkan_backend.create_buffer::<Mat4>(
            BufferDesc {
                size: size_of::<Mat4>() * CASCADE_COUNT,
                usage: BufferUsageFlags::UNIFORM,
                memory_hint: MemoryHint::CPUWritable,
            },
            None,
        );

        // LightingUbo: light_direction(Vec4) + light_color(Vec4) + ambient_light(Vec4) + cascade_depths(Vec4) = 4 * Vec4
        let lighting_buffer = vulkan_backend.create_buffer::<Vec4>(
            BufferDesc {
                size: size_of::<Vec4>() * 4,
                usage: BufferUsageFlags::UNIFORM,
                memory_hint: MemoryHint::CPUWritable,
            },
            None,
        );

        let shadow_sampler = vulkan_backend.create_sampler(SamplerDesc {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            address_u: SamplerAddressMode::ClampToEdge,
            address_v: SamplerAddressMode::ClampToEdge,
            address_w: SamplerAddressMode::ClampToEdge,
            compare_enable: true,
            compare_op: Some(CompareOp::Less),
        });

        // Shadow pass descriptor layout (set 0): cascade UBO + model SSBO
        let shadow_descriptor_layout =
            vulkan_backend.create_descriptor_layout(DescriptorLayoutDesc {
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
            });

        let shadow_descriptor_set =
            vulkan_backend.allocate_descriptor_set(shadow_descriptor_layout);
        vulkan_backend.update_descriptor_set(
            shadow_descriptor_set,
            &[
                DescriptorWriteDesc {
                    binding: 0,
                    value: DescriptorValue::UniformBuffer(cascade_buffer),
                },
                DescriptorWriteDesc {
                    binding: 1,
                    value: DescriptorValue::StorageBuffer(model_storage_buffer),
                },
            ],
        );

        // Lighting pass descriptor layout (set 0)
        let lighting_descriptor_layout =
            vulkan_backend.create_descriptor_layout(DescriptorLayoutDesc {
                bindings: vec![
                    DescriptorBinding {
                        binding: 0,
                        descriptor_type: DescriptorType::UniformBuffer,
                        count: 1,
                        stages: ShaderStage::FRAGMENT,
                    },
                    DescriptorBinding {
                        binding: 1,
                        descriptor_type: DescriptorType::CombinedImageSampler,
                        count: 1,
                        stages: ShaderStage::FRAGMENT,
                    },
                    DescriptorBinding {
                        binding: 2,
                        descriptor_type: DescriptorType::CombinedImageSampler,
                        count: 1,
                        stages: ShaderStage::FRAGMENT,
                    },
                    DescriptorBinding {
                        binding: 3,
                        descriptor_type: DescriptorType::CombinedImageSampler,
                        count: 1,
                        stages: ShaderStage::FRAGMENT,
                    },
                    DescriptorBinding {
                        binding: 4,
                        descriptor_type: DescriptorType::CombinedImageSampler,
                        count: 1,
                        stages: ShaderStage::FRAGMENT,
                    },
                    DescriptorBinding {
                        binding: 5,
                        descriptor_type: DescriptorType::CombinedImageSampler,
                        count: 1,
                        stages: ShaderStage::FRAGMENT,
                    },
                    DescriptorBinding {
                        binding: 6,
                        descriptor_type: DescriptorType::CombinedImageSampler,
                        count: 1,
                        stages: ShaderStage::FRAGMENT,
                    },
                    DescriptorBinding {
                        binding: 7,
                        descriptor_type: DescriptorType::UniformBuffer,
                        count: 1,
                        stages: ShaderStage::FRAGMENT,
                    },
                    DescriptorBinding {
                        binding: 8,
                        descriptor_type: DescriptorType::UniformBuffer,
                        count: 1,
                        stages: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    },
                ],
            });

        let lighting_descriptor_set =
            vulkan_backend.allocate_descriptor_set(lighting_descriptor_layout);

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

        let write = vec![
            DescriptorWriteDesc {
                binding: 0,
                value: DescriptorValue::UniformBuffer(camera_buffer),
            },
            DescriptorWriteDesc {
                binding: 1,
                value: DescriptorValue::StorageBuffer(model_storage_buffer),
            },
        ];

        vulkan_backend.update_descriptor_set(descriptor_handle, &write);

        Self {
            frame_images,
            camera_buffer,
            model_storage_buffer,
            descriptor_layout_handle,
            descriptor_handle,
            basic_sampler,
            cascade_buffer,
            lighting_buffer,
            shadow_sampler,
            shadow_descriptor_layout,
            shadow_descriptor_set,
            lighting_descriptor_layout,
            lighting_descriptor_set,
        }
    }
}

pub struct ResolutionSettings {
    pub window_resolution: Resolution,
    pub shadow_resolution: Resolution,
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
