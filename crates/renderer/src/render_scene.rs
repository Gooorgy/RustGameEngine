use material::material_manager::{MaterialVariant, ShaderPushConstantInfo};
use rendering_backend::backend_impl::resource_manager::MeshData;
use rendering_backend::buffer::BufferHandle;
use rendering_backend::descriptor::{DescriptorLayoutHandle, DescriptorSetHandle};

pub struct RenderScene {
    pub meshes: Vec<MeshRenderData>,
    pub camera: BufferHandle,
}

pub struct MeshRenderData {
    pub mesh_data: MeshData,
    pub material_data: MaterialData,
}

pub struct MaterialData {
    pub shader_variant: MaterialVariant,
    pub descriptor_set_handle: DescriptorSetHandle,
    pub descriptor_layout_handle: DescriptorLayoutHandle,
    pub push_constant_data: Vec<u8>,
}
