use rendering_backend::backend_impl::resource_manager::MeshData;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::buffer::BufferHandle;
use rendering_backend::descriptor::{
    DescriptorBinding, DescriptorLayoutDesc, DescriptorSetHandle, DescriptorType, DescriptorValue,
    ShaderStage,
};
use rendering_backend::image::{
    ImageAspect, ImageDesc, ImageHandle, ImageUsageFlags, TextureFormat,
};
use rendering_backend::pipeline::{
    BlendAttachmentDesc, BlendFactor, BlendOp, BlendStateDesc, ColorWriteMask, CompareOp, CullMode,
    DepthStencilDesc, FrontFace, PipelineDesc, PipelineHandle, PolygonMode, PushConstantDesc,
    RasterizationStateDesc, VertexInputDesc,
};
use rendering_backend::sampler::{Filter, SamplerAddressMode, SamplerDesc, SamplerHandle};
use std::mem::size_of;

pub struct GeometryRenderer {
    pipeline: PipelineHandle,
    descriptor_set: DescriptorSetHandle,
    texture: ImageHandle,
    sampler: SamplerHandle,
}

impl GeometryRenderer {
    pub fn new(
        vulkan_backend: &mut VulkanBackend,
        albedo_image: ImageHandle,
        depth_image: ImageHandle,
        camera_buffer: BufferHandle,
        model_storage_buffer: BufferHandle,
    ) -> GeometryRenderer {
        let texture_desc = ImageDesc {
            width: 1024,
            height: 1024,
            depth: 1,
            format: TextureFormat::R16g16b16a16Float,
            usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            aspect: ImageAspect::Color,
            mip_levels: 1,
            array_layers: 1,
            is_cubemap: false,
            clear_value: None,
        };

        let texture = vulkan_backend.create_image(texture_desc);
        let sampler = vulkan_backend.create_sampler(SamplerDesc {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            address_u: SamplerAddressMode::Repeat,
            address_v: SamplerAddressMode::Repeat,
            address_w: SamplerAddressMode::Repeat,
        });

        let layout_desc = DescriptorLayoutDesc {
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
                DescriptorBinding {
                    binding: 2,
                    descriptor_type: DescriptorType::CombinedImageSampler,
                    count: 1,
                    stages: ShaderStage::FRAGMENT,
                },
            ],
        };

        let descriptor_layout = vulkan_backend.create_descriptor_layout(layout_desc);
        let descriptor_set = vulkan_backend.allocate_descriptor_set(descriptor_layout);

        let pipeline_desc = PipelineDesc {
            vertex_shader: "vert".into(),
            fragment_shader: "frag".into(),
            color_attachments: vec![albedo_image],
            depth_attachment: Some(depth_image),
            layout: descriptor_layout,
            depth_stencil: DepthStencilDesc {
                depth_test_enable: true,
                depth_write_enable: true,
                depth_compare_op: CompareOp::Less,
                depth_bounds_test_enable: false,
                stencil_test_enable: false,
            },
            push_constant_ranges: vec![{
                PushConstantDesc {
                    offset: 0,
                    stages: ShaderStage::VERTEX,
                    size: size_of::<u64>(),
                }
            }],
            blend: BlendStateDesc {
                logic_op_enable: false,
                attachments: vec![
                    BlendAttachmentDesc {
                        color_write_mask: ColorWriteMask::ALL,
                        blend_enable: false,
                        src_color_blend: BlendFactor::One,
                        dst_color_blend: BlendFactor::Zero,
                        color_blend_op: BlendOp::Add,
                        src_alpha_blend: BlendFactor::One,
                        dst_alpha_blend: BlendFactor::Zero,
                        alpha_blend_op: BlendOp::Add,
                    },
                    // BlendAttachmentDesc {
                    //     color_write_mask: ColorWriteMask::ALL,
                    //     blend_enable: false,
                    //     src_color_blend: BlendFactor::One,
                    //     dst_color_blend: BlendFactor::Zero,
                    //     color_blend_op: BlendOp::Add,
                    //     src_alpha_blend: BlendFactor::One,
                    //     dst_alpha_blend: BlendFactor::Zero,
                    //     alpha_blend_op: BlendOp::Add,
                    // }
                ],
            },
            rasterization: RasterizationStateDesc {
                depth_clamp_enable: false,
                depth_bias_enable: false,
                discard_enable: false,
                polygon_mode: PolygonMode::Fill,
                cull_mode: CullMode::Back,
                front_face: FrontFace::CounterClockwise,
            },
            vertex_input: VertexInputDesc {
                bindings: vec![],
                attributes: vec![],
            },
        };

        let pipeline = vulkan_backend.create_graphics_pipeline(pipeline_desc);

        vulkan_backend.update_descriptor_set(
            descriptor_set,
            0,
            DescriptorValue::UniformBuffer(camera_buffer),
        );
        vulkan_backend.update_descriptor_set(
            descriptor_set,
            1,
            DescriptorValue::StorageBuffer(model_storage_buffer),
        );

        vulkan_backend.update_descriptor_set(
            descriptor_set,
            2,
            DescriptorValue::SampledImage {
                image: texture,
                sampler,
            },
        );

        Self {
            texture,
            pipeline,
            sampler,
            descriptor_set,
        }
    }

    pub fn render(
        &self,
        vulkan_backend: &mut VulkanBackend,
        scene_data: &Vec<MeshData>,
        albedo_image: ImageHandle,
        depth_image: ImageHandle,
    ) {
        vulkan_backend.begin_rendering(&[albedo_image], Some(&depth_image));

        vulkan_backend.bind_pipeline(self.pipeline);
        vulkan_backend.bind_descriptor_set(self.descriptor_set, self.pipeline);

        for (index, mesh) in scene_data.iter().enumerate() {
            vulkan_backend.update_push_constants(self.pipeline, ShaderStage::VERTEX, &[index as u32]);
            vulkan_backend.bind_vertex_buffer(mesh.vertex_buffer);
            vulkan_backend.bind_index_buffer(mesh.index_buffer);

            vulkan_backend.draw_indexed(mesh.index_count as u32, 0)
        }

        vulkan_backend.end_rendering();
    }
}
