use std::{ffi::CString, fs, io, path::Path};

const FRAGMENT_SHADER: &str = "frag";
const VERTEX_SHADER: &str = "vert";
const SHADER_PATH: &str = "./shaders/";
const SHADER_EXTENSION: &str = ".spv";

pub fn create(device: &ash::Device) {
    let vert_shader_code = read_shader_file(VERTEX_SHADER).expect("Unable to read vertex file");
    let frag_shader_code =
        read_shader_file(FRAGMENT_SHADER).expect("Unable to read fragment shader");

    let vert_shader_module = create_shader_module(&vert_shader_code, device);
    let frag_shader_module = create_shader_module(&frag_shader_code, device);

    let shader_name = CString::new("main").unwrap();

    let vert_shader_stage_info = ash::vk::PipelineShaderStageCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        stage: ash::vk::ShaderStageFlags::VERTEX,
        module: vert_shader_module,
        p_name: shader_name.as_ptr(),
        ..Default::default()
    };

    let frag_shader_stage_info = ash::vk::PipelineShaderStageCreateInfo {
        s_type: ash::vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        stage: ash::vk::ShaderStageFlags::FRAGMENT,
        module: frag_shader_module,
        p_name: shader_name.as_ptr(),
        ..Default::default()
    };

    let _shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

    unsafe {
        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);
    };
}

fn read_shader_file(shader_name: &str) -> Result<Vec<u8>, io::Error> {
    let path = Path::new(SHADER_PATH)
        .join(shader_name)
        .join(SHADER_EXTENSION);
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
