use crate::primitives::types::{EnginePrimitiveType, StaticMeshData};
use crate::traits::{GameObject, GameObjectType, HasGameObjectType};
use assets::MeshHandle;
use core::types::transform::Transform;
use core::EngineContext;
use legacy_macros::primitive_game_object;
use material::material_manager::MaterialHandle;
use std::cell::RefCell;
use std::rc::Rc;

#[primitive_game_object]
pub struct StaticMesh {
    object_type: GameObjectType,
    mesh_handle: MeshHandle,
    material_handle: Option<MaterialHandle>,
}

impl StaticMesh {
    pub fn new(mesh_handle: MeshHandle) -> Self {
        Self {
            object_type: GameObjectType::EnginePrimitive(EnginePrimitiveType::StaticMesh(
                StaticMeshData {
                    mesh_handle,
                    material_handle: None,
                },
            )),
            transform: Transform::default(),
            mesh_handle,
            material_handle: None,
        }
    }

    pub fn with_material(mut self, material: MaterialHandle) -> Self {
        self.material_handle = Some(material);
        self.object_type =
            GameObjectType::EnginePrimitive(EnginePrimitiveType::StaticMesh(StaticMeshData {
                material_handle: self.material_handle,
                mesh_handle: self.mesh_handle,
            }));

        self
    }
}

impl HasGameObjectType for StaticMesh {
    fn get_game_object_type(&self) -> GameObjectType {
        self.object_type
    }
}

impl GameObject for StaticMesh {
    fn setup(&mut self, engine_context: &EngineContext) {
        todo!()
    }
}
