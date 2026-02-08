use crate::frame_data::{FrameData, ResolutionSettings};
use nalgebra_glm::{Vec3, vec3};
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::descriptor::ShaderStage;
use rendering_backend::pipeline::CullMode::Front;
use rendering_backend::pipeline::FrontFace::CounterClockwise;
use rendering_backend::pipeline::PolygonMode::Fill;
use rendering_backend::pipeline::{
    CompareOp, DepthStencilDesc, PipelineDesc, PipelineHandle, PushConstantDesc,
    RasterizationStateDesc, VertexInputDesc,
};

pub struct LightingRenderer {
    pipeline: PipelineHandle,
}

impl LightingRenderer {
    pub fn new(mut vulkan_backend: &mut VulkanBackend, frame_data: FrameData) -> Self {
        let descriptor_layout = frame_data.descriptor_layout_handle;

        let pipeline_desc = PipelineDesc {
            vertex_shader: String::from("shadow"),
            fragment_shader: None,
            push_constant_ranges: vec![PushConstantDesc {
                stages: ShaderStage::VERTEX,
                offset: 0,
                size: size_of::<u64>(),
            }],
            layout: vec![descriptor_layout],
            color_attachments: vec![],
            depth_attachment: Some(
                *frame_data
                    .frame_images
                    .shadow_cascades
                    .first()
                    .expect("No shadow cascade image present!"),
            ),
            blend: None,
            depth_stencil: DepthStencilDesc {
                depth_test_enable: true,
                depth_write_enable: true,
                depth_compare_op: CompareOp::Less,
                depth_bounds_test_enable: false,
                stencil_test_enable: false,
            },
            rasterization: RasterizationStateDesc {
                cull_mode: Front,
                depth_bias_enable: false,
                depth_clamp_enable: true,
                discard_enable: false,
                front_face: CounterClockwise,
                polygon_mode: Fill,
            },
            vertex_input: VertexInputDesc {
                bindings: vec![],
                attributes: vec![],
            },
        };

        let pipeline = vulkan_backend.create_graphics_pipeline(pipeline_desc);

        Self { pipeline }
    }

    pub fn draw_frame() {}

    fn get_cascades(
        near: f32,
        far: f32,
        cascade_count: usize,
        resolution_settings: ResolutionSettings,
    ) {
        // let lambda = 0.9;
        // let mut splits = [0.0; cascade_count + 1];
        //
        // for i in 0..cascade_count {
        //     let idm = i as f32 / cascade_count as f32;
        //     let log = near * (far / near).powf(idm);
        //     let uniform = near + (far - near) * idm;
        //
        //     let split = log * lambda + uniform * (1.0 - lambda);
        //     splits[i] = split;
        // }
        //
        // for i in 0..cascade_count {
        //     let split_start = splits[i];
        //     let split_end = splits[i + 1];
        //
        //     let frustum_corners = [
        //         vec3(-1.0, -1.0, 0.0),
        //         vec3(1.0, -1.0, 0.0),
        //         vec3(1.0, 1.0, 0.0),
        //         vec3(-1.0, 1.0, 0.0),
        //         vec3(-1.0, -1.0, 1.0),
        //         vec3(1.0, -1.0, 1.0),
        //         vec3(1.0, 1.0, 1.0),
        //         vec3(-1.0, 1.0, 1.0),
        //     ];
        //
        //     let mut corners_world = [Vec3::zeros(); 8];
        //
        //     let aspect_ratio = resolution_settings.window_resolution.get_aspect_ratio();
        //
        //
        //     //let projection = nalgebra_glm::perspective(aspect_ratio, )
        //}
    }
}
