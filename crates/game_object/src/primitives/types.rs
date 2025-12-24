use assets::MeshHandle;
use material::material_manager::MaterialHandle;

#[derive(Clone, Copy)]
pub enum EnginePrimitiveType {
    StaticMesh(StaticMeshData),
    Unknown,
}

#[derive(Clone, Copy)]
pub struct StaticMeshData {
    pub mesh_handle: MeshHandle,
    pub material_handle: Option<MaterialHandle>,
}
