use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::buffer::BufferHandle;
use rendering_backend::descriptor::{
    DescriptorBinding, DescriptorLayoutDesc, DescriptorSetHandle, DescriptorType, DescriptorValue,
    ShaderStage,
};
use rendering_backend::image::ImageHandle;
use rendering_backend::pipeline::{
    BlendStateDesc, CompareOp, CullMode, DepthStencilDesc, FrontFace, PipelineDesc, PipelineHandle,
    PolygonMode, PushConstantDesc, RasterizationStateDesc, VertexInputDesc,
};
use std::mem::size_of;

pub struct LightingRenderer {
    pipeline: PipelineHandle,
    descriptor_set: DescriptorSetHandle,
}

impl LightingRenderer {
    pub fn new(
        vulkan_backend: &mut VulkanBackend,
        shadow_map: ImageHandle,
        model_storage_buffer: BufferHandle,
        cascade_buffer: BufferHandle,
    ) -> LightingRenderer {
        let descriptor_layout_desc = DescriptorLayoutDesc {
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

        let descriptor_layout = vulkan_backend.create_descriptor_layout(descriptor_layout_desc);
        let descriptor_set = vulkan_backend.allocate_descriptor_set(descriptor_layout);

        let pipeline_desc = PipelineDesc {
            vertex_shader: "shadow".into(),
            fragment_shader: None,
            rasterization: RasterizationStateDesc {
                depth_clamp_enable: true,
                depth_bias_enable: false,
                discard_enable: false,
                polygon_mode: PolygonMode::Fill,
                cull_mode: CullMode::Back,
                front_face: FrontFace::CounterClockwise,
            },
            push_constant_ranges: vec![
                PushConstantDesc {
                    offset: 0,
                    stages: ShaderStage::VERTEX,
                    size: size_of::<u64>(),
                },
                PushConstantDesc {
                    offset: 0,
                    stages: ShaderStage::VERTEX,
                    size: size_of::<u64>(),
                },
            ],
            depth_stencil: DepthStencilDesc {
                depth_test_enable: true,
                depth_write_enable: true,
                depth_compare_op: CompareOp::Less,
                depth_bounds_test_enable: false,
                stencil_test_enable: false,
            },
            vertex_input: VertexInputDesc {
                bindings: vec![],
                attributes: vec![],
            },
            blend: BlendStateDesc {
                logic_op_enable: false,
                attachments: vec![],
            },
            layout: descriptor_layout,
            depth_attachment: Some(shadow_map),
            color_attachments: vec![],
        };

        let pipeline = vulkan_backend.create_graphics_pipeline(pipeline_desc);

        vulkan_backend.update_descriptor_set(
            descriptor_set,
            0,
            DescriptorValue::UniformBuffer(cascade_buffer),
        );
        vulkan_backend.update_descriptor_set(
            descriptor_set,
            1,
            DescriptorValue::StorageBuffer(model_storage_buffer),
        );

        Self {
            descriptor_set,
            pipeline,
        }
    }

    // pub fn render(vulkan_backend: &mut VulkanBackend, shadow_map: ImageHandle) {
    //     for 0..SHADOW_CASCADE_COUNT
    //     vulkan_backend.begin_rendering(&[], Some(&shadow_map));
    //
    // }
}
