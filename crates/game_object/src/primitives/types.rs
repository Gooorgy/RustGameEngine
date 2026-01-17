use std::cell::RefCell;
use std::rc::Rc;
use assets::MeshHandle;
use material::material_manager::MaterialHandle;
use nalgebra_glm::Mat4;
use crate::primitives::camera::Camera;
use crate::primitives::static_mesh::StaticMesh;

#[derive(Clone, Copy)]
pub enum EnginePrimitiveType {
    StaticMesh(StaticMeshData),
    Camera(CameraData),
    Unknown,
}

#[derive(Clone, Copy)]
pub struct StaticMeshData {
    pub mesh_handle: MeshHandle,
    pub material_handle: Option<MaterialHandle>,
}

#[derive(Clone, Copy)]
pub struct CameraData {
    pub view: Mat4,
    pub projection: Mat4,

    pub near_clip: f32,
    pub far_clip: f32,

    pub fov: f32,
}