use crate::handle::Handle;

/// Compiled SPIR-V shader loaded from a `.spv` file.
/// Words are stored as u32 matching the SPIR-V binary layout,
/// ready to pass directly to vk::ShaderModuleCreateInfo.
#[derive(Clone, Debug)]
pub struct ShaderData {
    pub spv: Vec<u32>,
}

pub type ShaderHandle = Handle<ShaderData>;