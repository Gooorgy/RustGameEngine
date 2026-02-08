use crate::backend_impl::device::DeviceInfo;
use crate::backend_impl::resource_registry::ResourceRegistry;
use crate::backend_impl::vk_vertex_info::VulkanVertexInfo;
use crate::pipeline::PipelineDesc;
use ash::vk;
use ash::vk::{DynamicState, PipelineDynamicStateCreateInfo};
use std::{ffi::CString, fs, io, path::Path, ptr};

const SHADOW_SHADER: &str = "shadow";
const FRAGMENT_SHADER: &str = "frag";
const VERTEX_SHADER: &str = "vert";
const LIGHTING_SHADER: &str = "lighting";

const DEBUG_VERTEX: &str = "line_debug_vert";
const DEBUG_FRAGMENT: &str = "line_debug_frag";

const SHADER_PATH: &str = ".\\resources\\shaders";
const SHADER_EXTENSION: &str = ".spv";

#[derive(Clone)]

pub struct PipelineInfo {
    pub pipelines: Vec<vk::Pipeline>,
    pub pipeline_layout: vk::PipelineLayout,
}

impl PipelineInfo {
    pub fn create_pipeline_from_desc(
        device: &DeviceInfo,
        desc: PipelineDesc,
        resource_registry: &ResourceRegistry,
    ) -> Self {
        let vert_shader_code =
            Self::read_shader_file(&desc.vertex_shader).expect("Unable to read vertex file");

        let vert_shader_module =
            Self::create_shader_module(&vert_shader_code, &device.logical_device);

        let shader_name = CString::new("main").unwrap();

        let vert_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(&shader_name);

        let mut shader_stages = vec![vert_shader_stage_create_info];

        if let Some(fragment_shader) = desc.fragment_shader {
            let frag_shader_code =
                Self::read_shader_file(&fragment_shader).expect("Unable to read fragment shader");

            let frag_shader_module =
                Self::create_shader_module(&frag_shader_code, &device.logical_device);

            let frag_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(&shader_name);

            shader_stages.push(frag_shader_stage_create_info);
        }

        let dynamic_states = vec![DynamicState::VIEWPORT, DynamicState::SCISSOR];

        let dynamic_state_create_info =
            PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let vertex_binding_description = [VulkanVertexInfo::binding_description()];
        let vertex_attribute_description = VulkanVertexInfo::attribute_descriptions();

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
            .depth_clamp_enable(desc.rasterization.depth_clamp_enable)
            .depth_bias_enable(desc.rasterization.depth_bias_enable)
            .rasterizer_discard_enable(desc.rasterization.discard_enable)
            .polygon_mode(desc.rasterization.polygon_mode.into())
            .line_width(1f32)
            .cull_mode(desc.rasterization.cull_mode.into())
            .front_face(desc.rasterization.front_face.into());

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

        let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(desc.depth_stencil.depth_test_enable)
            .depth_write_enable(desc.depth_stencil.depth_write_enable)
            .depth_compare_op(desc.depth_stencil.depth_compare_op.into())
            .stencil_test_enable(desc.depth_stencil.stencil_test_enable);

        let set_layouts = desc
            .layout
            .iter()
            .map(|layout_handle| resource_registry.descriptor_layouts[layout_handle.0].layout)
            .collect::<Vec<_>>();

        let mut pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(set_layouts.as_slice());

        let push_constant_ranges = desc
            .push_constant_ranges
            .iter()
            .map(|range_desc| {
                vk::PushConstantRange::default()
                    .stage_flags(range_desc.stages.into())
                    .offset(range_desc.offset)
                    .size(range_desc.size as u32)
            })
            .collect::<Vec<_>>();

        if (push_constant_ranges.len() > 0) {
            pipeline_layout_create_info =
                pipeline_layout_create_info.push_constant_ranges(&push_constant_ranges);
        }

        let pipeline_layout = unsafe {
            device
                .logical_device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Unable to create pipeline layout")
        };

        let color_formats: Vec<vk::Format> = desc
            .color_attachments
            .iter()
            .map(|image_handle| resource_registry.images[image_handle.0].image_format)
            .collect();

        let depth_format = desc
            .depth_attachment
            .map(|image_handle| resource_registry.images[image_handle.0].image_format);

        let mut rendering_info =
            vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&color_formats);

        if let Some(df) = depth_format {
            rendering_info = rendering_info.depth_attachment_format(df);
        }

        let mut pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info_create_info)
            .input_assembly_state(&input_assembly_create_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterizer_create_info)
            .multisample_state(&multisampling_create_info)
            .dynamic_state(&dynamic_state_create_info)
            .layout(pipeline_layout)
            .depth_stencil_state(&depth_stencil_state_create_info)
            .push_next(&mut rendering_info);

        let mut color_blend_attachments = vec![];
        let mut color_blend_state_create_info = None;
        if let Some(blend) = desc.blend {
            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(blend.attachments[0].color_write_mask.into())
                .blend_enable(blend.attachments[0].blend_enable)
                .src_color_blend_factor(blend.attachments[0].src_color_blend.into())
                .dst_color_blend_factor(blend.attachments[0].dst_color_blend.into())
                .color_blend_op(blend.attachments[0].color_blend_op.into())
                .src_alpha_blend_factor(blend.attachments[0].src_alpha_blend.into())
                .dst_alpha_blend_factor(blend.attachments[0].dst_alpha_blend.into())
                .alpha_blend_op(blend.attachments[0].alpha_blend_op.into());

            color_blend_attachments.push(color_blend_attachment);

            color_blend_state_create_info = Some(
                vk::PipelineColorBlendStateCreateInfo::default()
                    .logic_op_enable(blend.logic_op_enable)
                    .logic_op(vk::LogicOp::COPY)
                    .attachments(color_blend_attachments.as_slice()),
            );
        };

        if let Some(color_blend_state_create_info) = &color_blend_state_create_info {
            pipeline_create_info =
                pipeline_create_info.color_blend_state(color_blend_state_create_info);
        }

        let graphics_pipelines = unsafe {
            device
                .logical_device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
                .expect("Unable to create graphics pipeline")
        };

        unsafe {
            device
                .logical_device
                .destroy_shader_module(vert_shader_module, None);
            // device
            //     .logical_device
            //     .destroy_shader_module(frag_shader_module, None);
        };

        Self {
            pipelines: graphics_pipelines,
            pipeline_layout,
        }
    }

    // pub fn create_line_pipeline(
    //     logical_device: &Device,
    //     desc_layout: &DescriptorSetLayout,
    // ) -> PipelineInfo {
    //     let vert_shader_code =
    //         Self::read_shader_file(DEBUG_VERTEX).expect("Unable to read vertex file");
    //     let frag_shader_code =
    //         Self::read_shader_file(DEBUG_FRAGMENT).expect("Unable to read fragment shader");
    //
    //     let vert_shader_module = Self::create_shader_module(&vert_shader_code, logical_device);
    //     let frag_shader_module = Self::create_shader_module(&frag_shader_code, logical_device);
    //
    //     let shader_name = CString::new("main").unwrap();
    //
    //     let vert_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
    //         .stage(vk::ShaderStageFlags::VERTEX)
    //         .module(vert_shader_module)
    //         .name(&shader_name);
    //
    //     let frag_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
    //         .stage(vk::ShaderStageFlags::FRAGMENT)
    //         .module(frag_shader_module)
    //         .name(&shader_name);
    //
    //     let dynamic_states = vec![DynamicState::VIEWPORT, DynamicState::SCISSOR];
    //
    //     let dynamic_state_create_info =
    //         PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    //
    //     let vertex_binding_description = Vertex::get_binding_descriptions();
    //     let vertex_attribute_description = Vertex::get_attribute_descriptions();
    //
    //     let vertex_input_info_create_info = vk::PipelineVertexInputStateCreateInfo::default()
    //         .vertex_attribute_descriptions(&vertex_attribute_description)
    //         .vertex_binding_descriptions(&vertex_binding_description);
    //
    //     let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo::default()
    //         .topology(vk::PrimitiveTopology::LINE_LIST)
    //         .primitive_restart_enable(false);
    //
    //     let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::default()
    //         .viewport_count(1)
    //         .scissor_count(1);
    //
    //     let rasterizer_create_info = vk::PipelineRasterizationStateCreateInfo::default()
    //         .depth_clamp_enable(false)
    //         .depth_bias_enable(false)
    //         .rasterizer_discard_enable(false)
    //         .polygon_mode(vk::PolygonMode::FILL)
    //         .line_width(1.0_f32)
    //         .cull_mode(vk::CullModeFlags::BACK)
    //         .front_face(vk::FrontFace::CLOCKWISE)
    //         .cull_mode(vk::CullModeFlags::BACK)
    //         .front_face(vk::FrontFace::COUNTER_CLOCKWISE);
    //
    //     let multisampling_create_info = vk::PipelineMultisampleStateCreateInfo {
    //         s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
    //         sample_shading_enable: vk::FALSE,
    //         rasterization_samples: vk::SampleCountFlags::TYPE_1,
    //         min_sample_shading: 1.0,
    //         p_sample_mask: ptr::null(),
    //         alpha_to_coverage_enable: vk::FALSE,
    //         alpha_to_one_enable: vk::FALSE,
    //         ..Default::default()
    //     };
    //

    //
    //     let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
    //         .color_write_mask(vk::ColorComponentFlags::RGBA)
    //         .blend_enable(false)
    //         .src_color_blend_factor(vk::BlendFactor::ONE)
    //         .dst_color_blend_factor(vk::BlendFactor::ZERO)
    //         .color_blend_op(vk::BlendOp::ADD)
    //         .src_alpha_blend_factor(vk::BlendFactor::ONE)
    //         .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
    //         .alpha_blend_op(vk::BlendOp::ADD);
    //
    //     let color_blend_attachments = [color_blend_attachment];
    //     let color_blending_create_info = PipelineColorBlendStateCreateInfo::default()
    //         .logic_op_enable(false)
    //         .logic_op(vk::LogicOp::COPY)
    //         .attachments(&color_blend_attachments);
    //
    //     let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
    //         .color_attachment_formats(&[ash::vk::Format::R16G16B16A16_SFLOAT]);
    //
    //     let shader_stages = [vert_shader_stage_create_info, frag_shader_stage_create_info];
    //
    //     let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default()
    //         .set_layouts(slice::from_ref(desc_layout))
    //         .push_constant_ranges(&push_constant_ranges);
    //
    //     let pipeline_layout = unsafe {
    //         logical_device
    //             .create_pipeline_layout(&pipeline_layout_create_info, None)
    //             .expect("Unable to create pipeline layout")
    //     };
    //
    //     let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
    //         .stages(&shader_stages)
    //         .input_assembly_state(&input_assembly_create_info)
    //         .viewport_state(&viewport_state_create_info)
    //         .rasterization_state(&rasterizer_create_info)
    //         .multisample_state(&multisampling_create_info)
    //         .color_blend_state(&color_blending_create_info)
    //         .dynamic_state(&dynamic_state_create_info)
    //         .subpass(0)
    //         .vertex_input_state(&vertex_input_info_create_info)
    //         .layout(pipeline_layout)
    //         .base_pipeline_handle(vk::Pipeline::null())
    //         .base_pipeline_index(-1)
    //         .push_next(&mut rendering_info);
    //
    //     let graphics_pipelines = unsafe {
    //         logical_device
    //             .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
    //             .expect("Unable to create graphics pipeline")
    //     };
    //
    //     unsafe {
    //         logical_device.destroy_shader_module(vert_shader_module, None);
    //         logical_device.destroy_shader_module(frag_shader_module, None);
    //     };
    //
    //     Self {
    //         pipelines: graphics_pipelines,
    //         pipeline_layout,
    //     }
    // }
    //
    // pub fn new(
    //     device: &DeviceInfo,
    //     desc: PipelineDesc,
    //     set_layouts: &[DescriptorSetLayout],
    // ) -> Self {
    //     let vert_shader_code =
    //         Self::read_shader_file(&*desc.vertex_shader).expect("Unable to read vertex file");
    //     let frag_shader_code =
    //         Self::read_shader_file(&*desc.fragment_shader).expect("Unable to read fragment shader");
    //
    //     let vert_shader_module =
    //         Self::create_shader_module(&vert_shader_code, &device.logical_device);
    //     let frag_shader_module =
    //         Self::create_shader_module(&frag_shader_code, &device.logical_device);
    //
    //     let shader_name = CString::new("main").unwrap();
    //
    //     let vert_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
    //         .stage(vk::ShaderStageFlags::VERTEX)
    //         .module(vert_shader_module)
    //         .name(&shader_name);
    //
    //     let frag_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
    //         .stage(vk::ShaderStageFlags::FRAGMENT)
    //         .module(frag_shader_module)
    //         .name(&shader_name);
    //
    //     let shader_stages = [vert_shader_stage_create_info, frag_shader_stage_create_info];
    //
    //     let dynamic_states = vec![
    //         DynamicState::VIEWPORT,
    //         DynamicState::SCISSOR,
    //         DynamicState::DEPTH_BIAS,
    //     ];
    //
    //     let dynamic_state_create_info =
    //         PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    //
    //     let vertex_binding_description = Vertex::get_binding_descriptions();
    //     let vertex_attribute_description = Vertex::get_attribute_descriptions();
    //
    //     let vertex_input_info_create_info = vk::PipelineVertexInputStateCreateInfo::default()
    //         .vertex_attribute_descriptions(&vertex_attribute_description)
    //         .vertex_binding_descriptions(&vertex_binding_description);
    //
    //     let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo::default()
    //         .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
    //         .primitive_restart_enable(false);
    //
    //     let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::default()
    //         .viewport_count(1)
    //         .scissor_count(1);
    //
    //     let rasterizer_create_info = vk::PipelineRasterizationStateCreateInfo::default()
    //         .depth_clamp_enable(false)
    //         .depth_bias_enable(false)
    //         .rasterizer_discard_enable(false)
    //         .polygon_mode(vk::PolygonMode::FILL)
    //         .line_width(1.0_f32)
    //         .cull_mode(vk::CullModeFlags::BACK)
    //         .front_face(vk::FrontFace::COUNTER_CLOCKWISE);
    //
    //     let multisampling_create_info = vk::PipelineMultisampleStateCreateInfo {
    //         s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
    //         sample_shading_enable: vk::FALSE,
    //         rasterization_samples: vk::SampleCountFlags::TYPE_1,
    //         min_sample_shading: 1.0,
    //         p_sample_mask: ptr::null(),
    //         alpha_to_coverage_enable: vk::FALSE,
    //         alpha_to_one_enable: vk::FALSE,
    //         ..Default::default()
    //     };
    //
    //     let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
    //         .color_write_mask(vk::ColorComponentFlags::RGBA)
    //         .blend_enable(false)
    //         .src_color_blend_factor(vk::BlendFactor::ONE)
    //         .dst_color_blend_factor(vk::BlendFactor::ZERO)
    //         .color_blend_op(vk::BlendOp::ADD)
    //         .src_alpha_blend_factor(vk::BlendFactor::ONE)
    //         .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
    //         .alpha_blend_op(vk::BlendOp::ADD);
    //
    //     let color_blend_attachments = [
    //         color_blend_attachment,
    //         color_blend_attachment,
    //         color_blend_attachment,
    //     ];
    //     let color_blending_create_info = PipelineColorBlendStateCreateInfo::default()
    //         .logic_op_enable(false)
    //         .logic_op(vk::LogicOp::COPY)
    //         .attachments(&color_blend_attachments);
    //
    //     let pipeline_layout_create_info =
    //         vk::PipelineLayoutCreateInfo::default().set_layouts(slice::from_ref(set_layout));
    //
    //     let pipeline_layout = unsafe {
    //         device
    //             .logical_device
    //             .create_pipeline_layout(&pipeline_layout_create_info, None)
    //             .expect("Unable to create pipeline layout")
    //     };
    //
    //     let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::default()
    //         .depth_test_enable(true)
    //         .depth_write_enable(true)
    //         .depth_compare_op(vk::CompareOp::LESS)
    //         .depth_bounds_test_enable(false)
    //         .stencil_test_enable(false);
    //
    //     let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
    //         .depth_attachment_format(vk::Format::D32_SFLOAT)
    //         .color_attachment_formats(&[
    //             vk::Format::R16G16B16A16_SFLOAT,
    //             vk::Format::R16G16B16A16_SFLOAT,
    //             vk::Format::R16G16B16A16_SFLOAT,
    //         ]);
    //
    //     let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
    //         .stages(&shader_stages)
    //         .vertex_input_state(&vertex_input_info_create_info)
    //         .input_assembly_state(&input_assembly_create_info)
    //         .viewport_state(&viewport_state_create_info)
    //         .rasterization_state(&rasterizer_create_info)
    //         .multisample_state(&multisampling_create_info)
    //         .color_blend_state(&color_blending_create_info)
    //         .dynamic_state(&dynamic_state_create_info)
    //         .layout(pipeline_layout)
    //         .subpass(0)
    //         .base_pipeline_handle(vk::Pipeline::null())
    //         .base_pipeline_index(-1)
    //         .depth_stencil_state(&depth_stencil_state_create_info)
    //         .push_next(&mut rendering_info);
    //
    //     let graphics_pipelines = unsafe {
    //         device
    //             .logical_device
    //             .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
    //             .expect("Unable to create graphics pipeline")
    //     };
    //
    //     unsafe {
    //         device
    //             .logical_device
    //             .destroy_shader_module(vert_shader_module, None);
    //         device
    //             .logical_device
    //             .destroy_shader_module(frag_shader_module, None);
    //     };
    //
    //     Self {
    //         pipelines: graphics_pipelines,
    //         pipeline_layout,
    //     }
    // }
    //
    // pub fn new_gbuffer_pipeline_old(
    //     logical_device: &ash::Device,
    //     set_layout: &vk::DescriptorSetLayout,
    // ) -> PipelineInfo {
    //     let vert_shader_code =
    //         Self::read_shader_file(VERTEX_SHADER).expect("Unable to read vertex file");
    //     let frag_shader_code =
    //         Self::read_shader_file(FRAGMENT_SHADER).expect("Unable to read fragment shader");
    //
    //     let vert_shader_module = Self::create_shader_module(&vert_shader_code, logical_device);
    //     let frag_shader_module = Self::create_shader_module(&frag_shader_code, logical_device);
    //
    //     let shader_name = CString::new("main").unwrap();
    //
    //     let vert_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
    //         .stage(vk::ShaderStageFlags::VERTEX)
    //         .module(vert_shader_module)
    //         .name(&shader_name);
    //
    //     let frag_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
    //         .stage(vk::ShaderStageFlags::FRAGMENT)
    //         .module(frag_shader_module)
    //         .name(&shader_name);
    //
    //     let shader_stages = [vert_shader_stage_create_info, frag_shader_stage_create_info];
    //
    //     let dynamic_states = vec![
    //         DynamicState::VIEWPORT,
    //         DynamicState::SCISSOR,
    //         DynamicState::DEPTH_BIAS,
    //     ];
    //
    //     let dynamic_state_create_info =
    //         PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    //
    //     let vertex_binding_description = Vertex::get_binding_descriptions();
    //     let vertex_attribute_description = Vertex::get_attribute_descriptions();
    //
    //     let vertex_input_info_create_info = vk::PipelineVertexInputStateCreateInfo::default()
    //         .vertex_attribute_descriptions(&vertex_attribute_description)
    //         .vertex_binding_descriptions(&vertex_binding_description);
    //
    //     let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo::default()
    //         .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
    //         .primitive_restart_enable(false);
    //
    //     let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::default()
    //         .viewport_count(1)
    //         .scissor_count(1);
    //
    //     let rasterizer_create_info = vk::PipelineRasterizationStateCreateInfo::default()
    //         .depth_clamp_enable(false)
    //         .depth_bias_enable(false)
    //         .rasterizer_discard_enable(false)
    //         .polygon_mode(vk::PolygonMode::FILL)
    //         .line_width(1.0_f32)
    //         .cull_mode(vk::CullModeFlags::BACK)
    //         .front_face(vk::FrontFace::COUNTER_CLOCKWISE);
    //
    //     let multisampling_create_info = vk::PipelineMultisampleStateCreateInfo {
    //         s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
    //         sample_shading_enable: vk::FALSE,
    //         rasterization_samples: vk::SampleCountFlags::TYPE_1,
    //         min_sample_shading: 1.0,
    //         p_sample_mask: ptr::null(),
    //         alpha_to_coverage_enable: vk::FALSE,
    //         alpha_to_one_enable: vk::FALSE,
    //         ..Default::default()
    //     };
    //
    //     let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
    //         .color_write_mask(vk::ColorComponentFlags::RGBA)
    //         .blend_enable(false)
    //         .src_color_blend_factor(vk::BlendFactor::ONE)
    //         .dst_color_blend_factor(vk::BlendFactor::ZERO)
    //         .color_blend_op(vk::BlendOp::ADD)
    //         .src_alpha_blend_factor(vk::BlendFactor::ONE)
    //         .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
    //         .alpha_blend_op(vk::BlendOp::ADD);
    //
    //     let color_blend_attachments = [
    //         color_blend_attachment,
    //         color_blend_attachment,
    //         color_blend_attachment,
    //     ];
    //     let color_blending_create_info = PipelineColorBlendStateCreateInfo::default()
    //         .logic_op_enable(false)
    //         .logic_op(vk::LogicOp::COPY)
    //         .attachments(&color_blend_attachments);
    //
    //     let pipeline_layout_create_info =
    //         vk::PipelineLayoutCreateInfo::default().set_layouts(slice::from_ref(set_layout));
    //
    //     let pipeline_layout = unsafe {
    //         logical_device
    //             .create_pipeline_layout(&pipeline_layout_create_info, None)
    //             .expect("Unable to create pipeline layout")
    //     };
    //
    //     let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::default()
    //         .depth_test_enable(true)
    //         .depth_write_enable(true)
    //         .depth_compare_op(vk::CompareOp::LESS)
    //         .depth_bounds_test_enable(false)
    //         .stencil_test_enable(false);
    //
    //     let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
    //         .depth_attachment_format(vk::Format::D32_SFLOAT)
    //         .color_attachment_formats(&[
    //             vk::Format::R16G16B16A16_SFLOAT,
    //             vk::Format::R16G16B16A16_SFLOAT,
    //             vk::Format::R16G16B16A16_SFLOAT,
    //         ]);
    //
    //     let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
    //         .stages(&shader_stages)
    //         .vertex_input_state(&vertex_input_info_create_info)
    //         .input_assembly_state(&input_assembly_create_info)
    //         .viewport_state(&viewport_state_create_info)
    //         .rasterization_state(&rasterizer_create_info)
    //         .multisample_state(&multisampling_create_info)
    //         .color_blend_state(&color_blending_create_info)
    //         .dynamic_state(&dynamic_state_create_info)
    //         .layout(pipeline_layout)
    //         .subpass(0)
    //         .base_pipeline_handle(vk::Pipeline::null())
    //         .base_pipeline_index(-1)
    //         .depth_stencil_state(&depth_stencil_state_create_info)
    //         .push_next(&mut rendering_info);
    //
    //     let graphics_pipelines = unsafe {
    //         logical_device
    //             .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
    //             .expect("Unable to create graphics pipeline")
    //     };
    //
    //     unsafe {
    //         logical_device.destroy_shader_module(vert_shader_module, None);
    //         logical_device.destroy_shader_module(frag_shader_module, None);
    //     };
    //
    //     Self {
    //         pipelines: graphics_pipelines,
    //         pipeline_layout,
    //     }
    // }
    //
    // pub fn new_lighing_pipeline(
    //     device: &DeviceInfo,
    //     set_layout: &DescriptorSetLayout,
    //     vert_shader_path: &str,
    //     frag_shader_path: &str,
    // ) -> PipelineInfo {
    //     let vert_shader_code =
    //         Self::read_shader_file(vert_shader_path).expect("Unable to read vertex file");
    //     let frag_shader_code =
    //         Self::read_shader_file(frag_shader_path).expect("Unable to read fragment shader");
    //
    //     let vert_shader_module =
    //         Self::create_shader_module(&vert_shader_code, &device.logical_device);
    //     let frag_shader_module =
    //         Self::create_shader_module(&frag_shader_code, &device.logical_device);
    //
    //     let shader_name = CString::new("main").unwrap();
    //
    //     let binding = [*set_layout];
    //     let pipeline_layout_create_info =
    //         vk::PipelineLayoutCreateInfo::default().set_layouts(&binding);
    //
    //     let pipeline_layout = unsafe {
    //         device
    //             .logical_device
    //             .create_pipeline_layout(&pipeline_layout_create_info, None)
    //             .expect("Unable to create pipeline layout")
    //     };
    //
    //     let frag_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
    //         .stage(vk::ShaderStageFlags::FRAGMENT)
    //         .module(frag_shader_module)
    //         .name(&shader_name);
    //
    //     let vert_shader_code =
    //         Self::read_shader_file("quad").expect("Unable to read vertex shader file");
    //
    //     let vert_shader_module =
    //         Self::create_shader_module(&vert_shader_code, &device.logical_device);
    //
    //     let vert_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
    //         .stage(vk::ShaderStageFlags::VERTEX)
    //         .module(vert_shader_module)
    //         .name(&shader_name);
    //
    //     let shader_stages = [vert_shader_stage_create_info, frag_shader_stage_create_info];
    //
    //     // Rasterizer discard (bypassing rasterization completely)
    //     let rasterizer_state = vk::PipelineRasterizationStateCreateInfo::default()
    //         .depth_clamp_enable(false)
    //         .rasterizer_discard_enable(false) // Key: Enable rasterizer discard
    //         .polygon_mode(vk::PolygonMode::FILL)
    //         .cull_mode(vk::CullModeFlags::NONE)
    //         .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
    //         .depth_bias_enable(false)
    //         .line_width(1.0);
    //
    //     // Color Blend State
    //     let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::default()
    //         .blend_enable(false)
    //         .color_write_mask(vk::ColorComponentFlags::RGBA);
    //
    //     let binding = [color_blend_attachment_state];
    //     let color_blend_state = PipelineColorBlendStateCreateInfo::default()
    //         .logic_op_enable(false)
    //         .attachments(&binding)
    //         .blend_constants([0.0, 0.0, 0.0, 0.0]);
    //
    //     // No multisampling for a fullscreen quad
    //     let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
    //         .rasterization_samples(vk::SampleCountFlags::TYPE_1)
    //         .sample_shading_enable(false)
    //         .min_sample_shading(1.0);
    //
    //     let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();
    //
    //     let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
    //         .topology(vk::PrimitiveTopology::TRIANGLE_LIST) // Defines primitive topology (e.g., triangles)
    //         .primitive_restart_enable(false); // No primitive restart
    //
    //     let dynamic_states = vec![
    //         DynamicState::VIEWPORT,
    //         DynamicState::SCISSOR,
    //         DynamicState::DEPTH_BIAS,
    //     ];
    //
    //     let dynamic_state_create_info =
    //         PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    //
    //     let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::default()
    //         .viewport_count(1)
    //         .scissor_count(1);
    //
    //     let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
    //         .color_attachment_formats(&[vk::Format::R16G16B16A16_SFLOAT]);
    //
    //     let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
    //         .stages(&shader_stages)
    //         .vertex_input_state(&vertex_input_state)
    //         .multisample_state(&multisample_state)
    //         .rasterization_state(&rasterizer_state)
    //         .color_blend_state(&color_blend_state)
    //         .input_assembly_state(&input_assembly_state)
    //         .dynamic_state(&dynamic_state_create_info)
    //         .viewport_state(&viewport_state_create_info)
    //         .layout(pipeline_layout)
    //         .push_next(&mut rendering_info);
    //
    //     let graphics_pipelines = unsafe {
    //         device
    //             .logical_device
    //             .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
    //             .expect("Unable to create graphics pipeline")
    //     };
    //
    //     unsafe {
    //         device
    //             .logical_device
    //             .destroy_shader_module(frag_shader_module, None);
    //     };
    //     Self {
    //         pipelines: graphics_pipelines,
    //         pipeline_layout,
    //     }
    // }
    //
    // pub fn create_shadow_map_pipeline(
    //     logical_device: &ash::Device,
    //     set_layout: &DescriptorSetLayout,
    // ) -> PipelineInfo {
    //     let shadow_shader_code =
    //         Self::read_shader_file(SHADOW_SHADER).expect("Unable to read vertex file");
    //
    //     let shadow_shader_module = Self::create_shader_module(&shadow_shader_code, logical_device);
    //
    //     let shader_name = CString::new("main").unwrap();
    //
    //     let vert_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
    //         .stage(vk::ShaderStageFlags::VERTEX)
    //         .module(shadow_shader_module)
    //         .name(&shader_name);
    //
    //     let shader_stages = [vert_shader_stage_create_info];
    //
    //     let dynamic_states = vec![DynamicState::VIEWPORT, DynamicState::SCISSOR];
    //
    //     let dynamic_state_create_info =
    //         PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    //
    //     let vertex_binding_description = Vertex::get_binding_descriptions();
    //     let vertex_attribute_description = Vertex::get_attribute_descriptions();
    //
    //     let vertex_input_info_create_info = vk::PipelineVertexInputStateCreateInfo::default()
    //         .vertex_attribute_descriptions(&vertex_attribute_description)
    //         .vertex_binding_descriptions(&vertex_binding_description);
    //
    //     let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo::default()
    //         .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
    //         .primitive_restart_enable(false);
    //
    //     let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::default()
    //         .viewport_count(1)
    //         .scissor_count(1);
    //
    //     let mut conservative_rasterization_create_info =
    //         vk::PipelineRasterizationConservativeStateCreateInfoEXT::default()
    //             .conservative_rasterization_mode(ConservativeRasterizationModeEXT::OVERESTIMATE)
    //             .extra_primitive_overestimation_size(0.0);
    //
    //     let rasterizer_create_info = vk::PipelineRasterizationStateCreateInfo::default()
    //         .depth_clamp_enable(true)
    //         .depth_bias_enable(false)
    //         .rasterizer_discard_enable(false)
    //         .polygon_mode(vk::PolygonMode::FILL)
    //         .line_width(1.0_f32)
    //         .cull_mode(vk::CullModeFlags::BACK)
    //         .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
    //         .push_next(&mut conservative_rasterization_create_info);
    //
    //     let multisampling_create_info = vk::PipelineMultisampleStateCreateInfo {
    //         s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
    //         sample_shading_enable: vk::FALSE,
    //         rasterization_samples: vk::SampleCountFlags::TYPE_1,
    //         min_sample_shading: 1.0,
    //         p_sample_mask: ptr::null(),
    //         alpha_to_coverage_enable: vk::FALSE,
    //         alpha_to_one_enable: vk::FALSE,
    //         ..Default::default()
    //     };
    //
    //     let push_constant_ranges = [PushConstantRange::default()
    //         .stage_flags(vk::ShaderStageFlags::VERTEX)
    //         .offset(0)
    //         .size(mem::size_of::<CascadeShadowPushConsts>() as u32)];
    //
    //     let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default()
    //         .set_layouts(slice::from_ref(set_layout))
    //         .push_constant_ranges(&push_constant_ranges);
    //
    //     let pipeline_layout = unsafe {
    //         logical_device
    //             .create_pipeline_layout(&pipeline_layout_create_info, None)
    //             .expect("Unable to create pipeline layout")
    //     };
    //
    //     let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::default()
    //         .depth_test_enable(true)
    //         .depth_write_enable(true)
    //         .depth_compare_op(vk::CompareOp::LESS)
    //         .depth_bounds_test_enable(false)
    //         .min_depth_bounds(0.0_f32)
    //         .max_depth_bounds(1.0_f32)
    //         .stencil_test_enable(false);
    //
    //     let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
    //         .depth_attachment_format(vk::Format::D32_SFLOAT);
    //
    //     let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
    //         .stages(&shader_stages)
    //         .vertex_input_state(&vertex_input_info_create_info)
    //         .input_assembly_state(&input_assembly_create_info)
    //         .viewport_state(&viewport_state_create_info)
    //         .rasterization_state(&rasterizer_create_info)
    //         .multisample_state(&multisampling_create_info)
    //         .dynamic_state(&dynamic_state_create_info)
    //         .layout(pipeline_layout)
    //         .subpass(0)
    //         .base_pipeline_handle(vk::Pipeline::null())
    //         .base_pipeline_index(-1)
    //         .depth_stencil_state(&depth_stencil_state_create_info)
    //         .push_next(&mut rendering_info);
    //
    //     let graphics_pipelines = unsafe {
    //         logical_device
    //             .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
    //             .expect("Unable to create graphics pipeline")
    //     };
    //
    //     unsafe {
    //         logical_device.destroy_shader_module(shadow_shader_module, None);
    //     };
    //
    //     Self {
    //         pipelines: graphics_pipelines,
    //         pipeline_layout,
    //     }
    // }

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
