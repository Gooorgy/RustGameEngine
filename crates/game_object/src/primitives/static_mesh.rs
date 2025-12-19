use assets::MeshHandle;
use core::EngineContext;
use core::types::transform::Transform;
use macros::primitive_game_object;
use crate::primitives::types::EnginePrimitiveType;
use crate::traits::{GameObject, GameObjectType, HasGameObjectType};
 
#[primitive_game_object]
pub struct StaticMesh {
    object_type: GameObjectType,
}

impl StaticMesh {
    pub fn new(mesh_handle: MeshHandle) -> Self {
        Self {
            object_type: GameObjectType::EnginePrimitive(EnginePrimitiveType::StaticMesh(mesh_handle)),
            transform: Transform::default(),
        }
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
