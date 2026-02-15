use crate::frame_data::FrameData;
use crate::render_scene::{MaterialData, RenderScene};
use material::material_manager::MaterialVariant;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::descriptor::ShaderStage;
use rendering_backend::pipeline::{
    BlendAttachmentDesc, BlendFactor, BlendOp, BlendStateDesc, ColorWriteMask, CompareOp, CullMode,
    DepthStencilDesc, FrontFace, PipelineDesc, PipelineHandle, PolygonMode, PushConstantDesc,
    RasterizationStateDesc, VertexInputDesc,
};
use std::collections::HashMap;

pub struct GeometryRenderer {
    pub pipeline_cache: HashMap<MaterialVariant, PipelineHandle>,
    // pub bound_pipeline_handle: Option<PipelineHandle>,
    // pub bound_descriptor_sets: Vec<DescriptorSetHandle>,
}

impl GeometryRenderer {
    pub fn new() -> Self {
        Self {
            pipeline_cache: HashMap::new(),
        }
    }

    pub fn draw_frame(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        render_scene: &RenderScene,
        frame_data: &FrameData,
    ) {
        vulkan_backend.begin_rendering(
            &[frame_data.frame_images.gbuffer_albedo, frame_data.frame_images.gbuffer_normal],
            Some(&frame_data.frame_images.gbuffer_depth),
        );

        for (index, mesh_data) in render_scene.meshes.iter().enumerate() {
            let pipeline =
                self.get_or_create_pipeline(vulkan_backend, frame_data, &mesh_data.material_data);

            vulkan_backend.bind_pipeline(pipeline);
            vulkan_backend.bind_descriptor_sets(
                &[
                    frame_data.descriptor_handle,
                    mesh_data.material_data.descriptor_set_handle,
                ],
                pipeline,
            );

            vulkan_backend.update_push_constants(pipeline, ShaderStage::VERTEX, &[index]);
            vulkan_backend.update_push_constants_raw(
                pipeline,
                ShaderStage::FRAGMENT,
                mesh_data.material_data.push_constant_data.as_slice(),
                16, //size_of::<u32>() as u32,
            );

            vulkan_backend.bind_vertex_buffer(mesh_data.mesh_data.vertex_buffer);
            vulkan_backend.bind_index_buffer(mesh_data.mesh_data.index_buffer);

            vulkan_backend.draw_indexed(mesh_data.mesh_data.index_count as u32, 0);
        }

        vulkan_backend.end_rendering();
    }

    fn get_or_create_pipeline(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        frame_data: &FrameData,
        material_data: &MaterialData,
    ) -> PipelineHandle {
        let descriptors = self.pipeline_cache.get(&material_data.shader_variant);
        if let Some(pipeline) = descriptors {
            return *pipeline;
        }

        println!("Pipeline not found... Creating new");

        let pipeline_desc = PipelineDesc {
            vertex_shader: "vert".into(),
            fragment_shader: Some(material_data.shader_variant.name.clone()),
            color_attachments: vec![frame_data.frame_images.gbuffer_albedo, frame_data.frame_images.gbuffer_normal],
            depth_attachment: Some(frame_data.frame_images.gbuffer_depth),
            layout: vec![
                frame_data.descriptor_layout_handle,
                material_data.descriptor_layout_handle,
            ],
            depth_stencil: DepthStencilDesc {
                depth_test_enable: true,
                depth_write_enable: true,
                depth_compare_op: CompareOp::Less,
                depth_bounds_test_enable: false,
                stencil_test_enable: false,
            },
            push_constant_ranges: vec![
                PushConstantDesc {
                    offset: 0,
                    stages: ShaderStage::VERTEX,
                    size: size_of::<u64>(),
                },
                PushConstantDesc {
                    offset: 16, //size_of::<u32>() as u32,
                    stages: ShaderStage::FRAGMENT,
                    size: material_data.push_constant_data.len(),
                },
            ],
            blend: Some(BlendStateDesc {
                logic_op_enable: false,
                attachments: vec![BlendAttachmentDesc {
                    color_write_mask: ColorWriteMask::ALL,
                    blend_enable: false,
                    src_color_blend: BlendFactor::One,
                    dst_color_blend: BlendFactor::Zero,
                    color_blend_op: BlendOp::Add,
                    src_alpha_blend: BlendFactor::One,
                    dst_alpha_blend: BlendFactor::Zero,
                    alpha_blend_op: BlendOp::Add,
                },
                                  BlendAttachmentDesc {
                                      color_write_mask: ColorWriteMask::ALL,
                                      blend_enable: false,
                                      src_color_blend: BlendFactor::One,
                                      dst_color_blend: BlendFactor::Zero,
                                      color_blend_op: BlendOp::Add,
                                      src_alpha_blend: BlendFactor::One,
                                      dst_alpha_blend: BlendFactor::Zero,
                                      alpha_blend_op: BlendOp::Add,
                                  }],
            }),
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

        let pipeline_handle = vulkan_backend.create_graphics_pipeline(pipeline_desc);
        self.pipeline_cache
            .insert(material_data.shader_variant.clone(), pipeline_handle);

        pipeline_handle
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct GeometryPipelineKey {
    pub shader_path: String,
}
