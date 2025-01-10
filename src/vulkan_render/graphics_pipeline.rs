use std::{ffi::CString, fs, io, path::Path, ptr, slice};

use super::structs::Vertex;
use ash::vk;
use ash::vk::{DynamicState, PipelineColorBlendStateCreateInfo, PipelineDynamicStateCreateInfo};

const FRAGMENT_SHADER: &str = "frag";
const VERTEX_SHADER: &str = "vert";
const LIGHTING_SHADER: &str = "lighting";
const SHADER_PATH: &str = ".\\resources\\shaders";
const SHADER_EXTENSION: &str = ".spv";

pub struct PipelineInfo {
    pub pipelines: Vec<vk::Pipeline>,
    pub pipeline_layout: vk::PipelineLayout,
}

impl PipelineInfo {
    pub fn new_gbuffer_pipeline(
        logical_device: &ash::Device,
        set_layout: &vk::DescriptorSetLayout,
    ) -> PipelineInfo {
        let vert_shader_code =
            Self::read_shader_file(VERTEX_SHADER).expect("Unable to read vertex file");
        let frag_shader_code =
            Self::read_shader_file(FRAGMENT_SHADER).expect("Unable to read fragment shader");

        let vert_shader_module = Self::create_shader_module(&vert_shader_code, logical_device);
        let frag_shader_module = Self::create_shader_module(&frag_shader_code, logical_device);

        let shader_name = CString::new("main").unwrap();

        let vert_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(&shader_name);

        let frag_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(&shader_name);

        let shader_stages = [vert_shader_stage_create_info, frag_shader_stage_create_info];

        let dynamic_states = vec![DynamicState::VIEWPORT, DynamicState::SCISSOR];

        let dynamic_state_create_info =
            PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let vertex_binding_description = Vertex::get_binding_descriptions();
        let vertex_attribute_description = Vertex::get_attribute_descriptions();

        let vertex_input_info_create_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&vertex_attribute_description)
            .vertex_binding_descriptions(&vertex_binding_description);

        let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterizer_create_info = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .depth_bias_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0_f32)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let multisampling_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.0,
            p_sample_mask: ptr::null(),
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable: vk::FALSE,
            ..Default::default()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blend_attachments = [color_blend_attachment];
        let color_blending_create_info = PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments);

        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(slice::from_ref(set_layout));

        let pipeline_layout = unsafe {
            logical_device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Unable to create pipeline layout")
        };

        let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0_f32)
            .max_depth_bounds(1.0_f32)
            .stencil_test_enable(false);

        let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
            .depth_attachment_format(vk::Format::D32_SFLOAT)
            .color_attachment_formats(&[vk::Format::R16G16B16A16_SFLOAT]);

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info_create_info)
            .input_assembly_state(&input_assembly_create_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterizer_create_info)
            .multisample_state(&multisampling_create_info)
            .color_blend_state(&color_blending_create_info)
            .dynamic_state(&dynamic_state_create_info)
            .layout(pipeline_layout)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .depth_stencil_state(&depth_stencil_state_create_info)
            .push_next(&mut rendering_info);

        let graphics_pipelines = unsafe {
            logical_device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
                .expect("Unable to create graphics pipeline")
        };

        unsafe {
            logical_device.destroy_shader_module(vert_shader_module, None);
            logical_device.destroy_shader_module(frag_shader_module, None);
        };

        Self {
            pipelines: graphics_pipelines,
            pipeline_layout,
        }
    }

    pub fn new_lighing_pipeline(
        logical_device: &ash::Device,
        set_layout: &vk::DescriptorSetLayout,
    ) -> PipelineInfo {
        let binding = [*set_layout];
        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&binding);

        let pipeline_layout = unsafe {
            logical_device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Unable to create pipeline layout")
        };

        let lighting_shader_code =
            Self::read_shader_file(LIGHTING_SHADER).expect("Unable to read lighting file");

        let lighting_shader_module =
            Self::create_shader_module(&lighting_shader_code, logical_device);

        let shader_name = CString::new("main").unwrap();

        let frag_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(lighting_shader_module)
            .name(&shader_name);

        let vert_shader_code =
            Self::read_shader_file("quad").expect("Unable to read vertex shader file");

        let vert_shader_module = Self::create_shader_module(&vert_shader_code, logical_device);

        let vert_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(&shader_name);

        let shader_stages = [vert_shader_stage_create_info, frag_shader_stage_create_info];

        // Rasterizer discard (bypassing rasterization completely)
        let rasterizer_state = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false) // Key: Enable rasterizer discard
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0);

        // Color Blend State
        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA);

        let binding = [color_blend_attachment_state];
        let color_blend_state = PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&binding)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        // No multisampling for a fullscreen quad
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(1.0);

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST) // Defines primitive topology (e.g., triangles)
            .primitive_restart_enable(false); // No primitive restart

        let dynamic_states = vec![
            DynamicState::VIEWPORT,
            DynamicState::SCISSOR,
            DynamicState::DEPTH_BIAS,
        ];

        let dynamic_state_create_info =
            PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&[vk::Format::R16G16B16A16_SFLOAT]);

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .multisample_state(&multisample_state)
            .rasterization_state(&rasterizer_state)
            .color_blend_state(&color_blend_state)
            .input_assembly_state(&input_assembly_state)
            .dynamic_state(&dynamic_state_create_info)
            .viewport_state(&viewport_state_create_info)
            .layout(pipeline_layout)
            .push_next(&mut rendering_info);

        let graphics_pipelines = unsafe {
            logical_device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
                .expect("Unable to create graphics pipeline")
        };

        unsafe {
            logical_device.destroy_shader_module(lighting_shader_module, None);
        };
        Self {
            pipelines: graphics_pipelines,
            pipeline_layout,
        }
    }

    fn read_shader_file(shader_name: &str) -> Result<Vec<u8>, io::Error> {
        let path = Path::new(SHADER_PATH).join(format!("{}{}", shader_name, SHADER_EXTENSION));

        println!("{:?}", path);
        fs::read(path)
    }

    fn create_shader_module(code: &[u8], device: &ash::Device) -> vk::ShaderModule {
        unsafe {
            let (_prefix, shorts, _suffix) = code.align_to::<u32>();
            let create_info = vk::ShaderModuleCreateInfo::default().code(shorts);
            device
                .create_shader_module(&create_info, None)
                .expect("Unable to create shader module")
        }
    }
}
