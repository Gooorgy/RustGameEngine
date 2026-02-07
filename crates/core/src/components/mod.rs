mod SpectatorCamera;

use crate::types::transform::Transform;
use assets::{MaterialHandle, MeshHandle};
use ecs::component::Component;
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

pub struct MeshComponent {
    pub mesh_handle: MeshHandle,
    pub material_handle: Option<MaterialHandle>,
}

impl MeshComponent {
    pub fn new(mesh_handle: MeshHandle) -> Self {
        Self {
            mesh_handle,
            material_handle: None,
        }
    }

    pub fn with_material(mut self, material: MaterialHandle) -> Self {
        self.material_handle = Some(material);

        self
    }
}
