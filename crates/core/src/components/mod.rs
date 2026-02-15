use crate::types::transform::Transform;
use assets::MeshHandle;
use ecs::component::Component;
use material::material_manager::MaterialHandle;
use nalgebra_glm::Vec3;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Component, Default)]
pub struct TransformComponent(pub Transform);

impl Deref for TransformComponent {
    type Target = Transform;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TransformComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Debug, Component)]
pub struct MeshComponent {
    pub mesh_handle: MeshHandle,
}

impl MeshComponent {
    pub fn new(mesh_handle: MeshHandle) -> Self {
        Self { mesh_handle }
    }
}

#[derive(Clone, Component)]
pub struct MaterialComponent {
    pub material_handle: MaterialHandle,
}

impl MaterialComponent {
    pub fn new(material_handle: MaterialHandle) -> Self {
        Self { material_handle }
    }
}

#[derive(Clone, Debug, Component)]
pub struct CameraComponent {
    pub near_clip: f32,
    pub far_clip: f32,
    pub fov: f32,
    pub active: bool,
}

#[derive(Component, Debug, Clone)]
pub struct CameraControllerComponent {
    pub speed: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl CameraControllerComponent {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Clone, Debug, Component)]
pub struct DirectionalLightComponent {
    pub color: Vec3,
    pub intensity: f32,
    pub ambient_color: Vec3,
    pub ambient_intensity: f32,
}
