use glm::Vec4;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MaterialInstanceData {
    pub base_color: Vec4,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub emissive_color: Vec4,
    pub texture_indices: [i32; 4],
}