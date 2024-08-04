use std::{ffi::CString, fs, io, path::Path, ptr};

const FRAGMENT_SHADER: &str = "frag";
const VERTEX_SHADER: &str = "vert";
const SHADER_PATH: &str = ".\\src\\shaders";
const SHADER_EXTENSION: &str = ".spv";

pub fn create(
    device: &ash::Device,
    render_pass: &ash::vk::RenderPass,
) -> (ash::vk::PipelineLayout, ash::vk::Pipeline) {
    let vert_shader_code = read_shader_file(VERTEX_SHADER).expect("Unable to read vertex file");
    let frag_shader_code =
        read_shader_file(FRAGMENT_SHADER).expect("Unable to read fragment shader");

    let vert_shader_module = create_shader_module(&vert_shader_code, device);
    let frag_shader_module = create_shader_module(&frag_shader_code, device);

    let shader_name = CString::new("main").unwrap();

    let vert_shader_stage_create_info = ash::vk::PipelineShaderStageCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        stage: ash::vk::ShaderStageFlags::VERTEX,
        module: vert_shader_module,
        p_name: shader_name.as_ptr(),
        ..Default::default()
    };

    let frag_shader_stage_create_info = ash::vk::PipelineShaderStageCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        stage: ash::vk::ShaderStageFlags::FRAGMENT,
        module: frag_shader_module,
        p_name: shader_name.as_ptr(),
        ..Default::default()
    };

    let shader_stages = [vert_shader_stage_create_info, frag_shader_stage_create_info];

    let dynamic_states = vec![
        ash::vk::DynamicState::VIEWPORT,
        ash::vk::DynamicState::SCISSOR,
    ];

    let dynamic_state_create_info = ash::vk::PipelineDynamicStateCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        dynamic_state_count: dynamic_states.len() as u32,
        p_dynamic_states: dynamic_states.as_ptr(),
        ..Default::default()
    };

    let vertex_input_info_create_info = ash::vk::PipelineVertexInputStateCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: ptr::null(),
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: ptr::null(),
        ..Default::default()
    };

    let input_assembly_create_info = ash::vk::PipelineInputAssemblyStateCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        topology: ash::vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: ash::vk::FALSE,
        ..Default::default()
    };

    let viewport_state_create_info = ash::vk::PipelineViewportStateCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        viewport_count: 1,
        scissor_count: 1,
        ..Default::default()
    };

    let rasterizer_create_info = ash::vk::PipelineRasterizationStateCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        depth_clamp_enable: ash::vk::FALSE,
        rasterizer_discard_enable: ash::vk::FALSE,
        polygon_mode: ash::vk::PolygonMode::FILL,
        line_width: 1.0_f32,
        cull_mode: ash::vk::CullModeFlags::BACK,
        front_face: ash::vk::FrontFace::CLOCKWISE,
        depth_bias_enable: ash::vk::FALSE,
        ..Default::default()
    };

    let multisampling_create_info = ash::vk::PipelineMultisampleStateCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        sample_shading_enable: ash::vk::FALSE,
        rasterization_samples: ash::vk::SampleCountFlags::TYPE_1,
        min_sample_shading: 1.0,
        p_sample_mask: ptr::null(),
        alpha_to_coverage_enable: ash::vk::FALSE,
        alpha_to_one_enable: ash::vk::FALSE,
        ..Default::default()
    };

    let color_blend_attachment = ash::vk::PipelineColorBlendAttachmentState {
        color_write_mask: ash::vk::ColorComponentFlags::RGBA,
        blend_enable: ash::vk::FALSE,
        src_color_blend_factor: ash::vk::BlendFactor::ONE,
        dst_color_blend_factor: ash::vk::BlendFactor::ZERO,
        color_blend_op: ash::vk::BlendOp::ADD,
        src_alpha_blend_factor: ash::vk::BlendFactor::ONE,
        dst_alpha_blend_factor: ash::vk::BlendFactor::ZERO,
        alpha_blend_op: ash::vk::BlendOp::ADD,
        ..Default::default()
    };

    let color_blending_create_info = ash::vk::PipelineColorBlendStateCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        logic_op_enable: ash::vk::FALSE,
        logic_op: ash::vk::LogicOp::COPY,
        attachment_count: 1,
        p_attachments: &color_blend_attachment,
        ..Default::default()
    };

    let pipeline_layout_create_info = ash::vk::PipelineLayoutCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,

        ..Default::default()
    };

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("Unable to create pipeline layout")
    };

    let pipeline_create_info = ash::vk::GraphicsPipelineCreateInfo {
        s_type: ash::vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        stage_count: 2,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_info_create_info,
        p_input_assembly_state: &input_assembly_create_info,
        p_viewport_state: &viewport_state_create_info,
        p_rasterization_state: &rasterizer_create_info,
        p_multisample_state: &multisampling_create_info,
        p_depth_stencil_state: ptr::null(),
        p_color_blend_state: &color_blending_create_info,
        p_dynamic_state: &dynamic_state_create_info,
        layout: pipeline_layout,
        render_pass: *render_pass,
        subpass: 0,
        base_pipeline_handle: ash::vk::Pipeline::null(),
        base_pipeline_index: -1,
        ..Default::default()
    };

    let graphics_pipeline = unsafe {
        device
            .create_graphics_pipelines(
                ash::vk::PipelineCache::null(),
                &[pipeline_create_info],
                None,
            )
            .expect("Unable to create graphics pipeline")
    };

    unsafe {
        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);
    };

    (pipeline_layout, graphics_pipeline[0])
}

fn read_shader_file(shader_name: &str) -> Result<Vec<u8>, io::Error> {
    let path = Path::new(SHADER_PATH).join(format!("{}{}", shader_name, SHADER_EXTENSION));

    println!("{:?}", path);
    fs::read(path)
}

fn create_shader_module(code: &Vec<u8>, device: &ash::Device) -> ash::vk::ShaderModule {
    let create_info = ash::vk::ShaderModuleCreateInfo {
        s_type: ash::vk::StructureType::SHADER_MODULE_CREATE_INFO,
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
        ..Default::default()
    };

    unsafe {
        device
            .create_shader_module(&create_info, None)
            .expect("Unable to create shader module")
    }
}
