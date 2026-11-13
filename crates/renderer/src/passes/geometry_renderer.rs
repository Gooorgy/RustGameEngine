use crate::frame_data::FrameData;
use crate::render_scene::{MaterialData, RenderScene};
use crate::shader_loader::ShaderCache;
use material::material_manager::{MaterialManager, MaterialVariantHandle};
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::descriptor::ShaderStage;
use rendering_backend::pipeline::{
    BlendAttachmentDesc, BlendFactor, BlendOp, BlendStateDesc, ColorWriteMask, CompareOp, CullMode,
    DepthStencilDesc, FrontFace, PipelineDesc, PipelineHandle, PolygonMode, PrimitiveTopology,
    PushConstantDesc, RasterizationStateDesc, VertexInputDesc, get_mesh_vertex_info,
};
use std::collections::HashMap;

/// Byte offset of the fragment push constant block. The vertex block occupies
/// bytes 0-7 (one u64 mesh index), but Vulkan push constant ranges must be
/// 16-byte aligned, so the fragment block starts here.
const FRAGMENT_PUSH_CONSTANT_OFFSET: u32 = 16;

pub struct GeometryRenderer {
    pub pipeline_cache: HashMap<
        (
            MaterialVariantHandle,
            rendering_backend::descriptor::DescriptorLayoutHandle,
        ),
        PipelineHandle,
    >,
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
        shader_cache: &mut ShaderCache,
        material_manager: &MaterialManager,
    ) {
        profiling::scope!("geometry_frame");

        vulkan_backend.begin_rendering(
            &[
                frame_data.frame_images.gbuffer_albedo,
                frame_data.frame_images.gbuffer_normal,
            ],
            Some(&frame_data.frame_images.gbuffer_depth),
        );

        for (index, mesh_data) in render_scene.meshes.iter().enumerate() {
            let pipeline = self.get_or_create_pipeline(
                vulkan_backend,
                frame_data,
                &mesh_data.material_data,
                shader_cache,
                material_manager,
            );

            vulkan_backend.bind_pipeline(pipeline);
            vulkan_backend.bind_descriptor_sets(
                &[
                    frame_data.descriptor_handle,
                    mesh_data.material_data.descriptor_set_handle,
                ],
                pipeline,
            );

            vulkan_backend.update_push_constants(pipeline, ShaderStage::VERTEX, &[index]);

            if !mesh_data.material_data.push_constant_data.is_empty() {
                vulkan_backend.update_push_constants_raw(
                    pipeline,
                    ShaderStage::FRAGMENT,
                    mesh_data.material_data.push_constant_data.as_slice(),
                    FRAGMENT_PUSH_CONSTANT_OFFSET,
                );
            }

            vulkan_backend.bind_vertex_buffer(mesh_data.mesh_data.vertex_buffer);
            vulkan_backend.bind_index_buffer(mesh_data.mesh_data.index_buffer);
            vulkan_backend.draw_indexed(mesh_data.mesh_data.index_count as u32, 0, 0);
        }

        vulkan_backend.end_rendering();
    }

    fn get_or_create_pipeline(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        frame_data: &FrameData,
        material_data: &MaterialData,
        shader_cache: &mut ShaderCache,
        material_manager: &MaterialManager,
    ) -> PipelineHandle {
        let cache_key = (
            material_data.variant_handle,
            material_data.descriptor_layout_handle,
        );
        if let Some(&pipeline) = self.pipeline_cache.get(&cache_key) {
            return pipeline;
        }

        let variant = material_manager.get_variant(material_data.variant_handle);
        let vert_bytes = shader_cache.load(
            &variant.vertex_shader,
            &[], // vertex shader permutations are encoded in the name for built-ins
        );
        let frag_bytes = shader_cache.load(&variant.fragment_shader, &variant.active_defines);

        let mut push_constant_ranges = vec![PushConstantDesc {
            offset: 0,
            stages: ShaderStage::VERTEX,
            size: size_of::<u64>(),
        }];
        if variant.push_constant_size > 0 {
            push_constant_ranges.push(PushConstantDesc {
                offset: FRAGMENT_PUSH_CONSTANT_OFFSET,
                stages: ShaderStage::FRAGMENT,
                size: variant.push_constant_size,
            });
        }

        let pipeline_desc = PipelineDesc {
            vertex_shader: vert_bytes,
            fragment_shader: Some(frag_bytes),
            color_attachments: vec![
                frame_data.frame_images.gbuffer_albedo,
                frame_data.frame_images.gbuffer_normal,
            ],
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
            push_constant_ranges,
            blend: Some(BlendStateDesc {
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
                ],
            }),
            rasterization: RasterizationStateDesc {
                depth_clamp_enable: false,
                depth_bias_enable: false,
                discard_enable: false,
                polygon_mode: PolygonMode::Fill,
                cull_mode: CullMode::Back,
                front_face: FrontFace::CounterClockwise,
            },
            vertex_input: get_mesh_vertex_info(),
            topology: PrimitiveTopology::TriangleList,
        };

        let pipeline_handle = vulkan_backend.create_graphics_pipeline(pipeline_desc);
        self.pipeline_cache.insert(cache_key, pipeline_handle);

        pipeline_handle
    }
}
