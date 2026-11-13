use crate::render_data::{CameraRenderData, DirectionalLightData, PointLightData};
use material::material_manager::MaterialVariantHandle;
use rendering_backend::backend_impl::resource_manager::GpuMeshData;
use rendering_backend::descriptor::{DescriptorLayoutHandle, DescriptorSetHandle};

pub struct RenderScene {
    pub meshes: Vec<MeshRenderData>,
    pub camera_data: Option<CameraRenderData>,
    pub directional_light: Option<DirectionalLightData>,
    pub point_lights: Vec<PointLightData>
}

pub struct MeshRenderData {
    pub mesh_data: GpuMeshData,
    pub material_data: MaterialData,
}

pub struct MaterialData {
    pub variant_handle: MaterialVariantHandle,
    pub descriptor_set_handle: DescriptorSetHandle,
    pub descriptor_layout_handle: DescriptorLayoutHandle,
    pub push_constant_data: Vec<u8>,
}
