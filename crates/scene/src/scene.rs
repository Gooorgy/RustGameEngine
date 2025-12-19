use crate::registration::GameObjectRegistration;
use assets::MeshHandle;
use game_object::primitives::types::EnginePrimitiveType;
use game_object::traits::{GameObject, GameObjectType};
use core::types::transform::Transform;

pub struct SceneManager {
    pub game_objects: Vec<GameObjectRegistration>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            game_objects: Vec::new(),
        }
    }

    pub fn register_game_object(
        &mut self,
        component: impl GameObject + 'static,
    ) -> &GameObjectRegistration {
        let registration = GameObjectRegistration::new(component);
        self.add_registration(registration)
    }

    fn add_registration(
        &mut self,
        registration: GameObjectRegistration,
    ) -> &GameObjectRegistration {
        self.game_objects.push(registration);

        println!("Adding component registration");
        &self.game_objects[self.game_objects.len() - 1]
    }

    pub fn get_static_meshes(&mut self) -> Vec<(MeshHandle, Transform)> {
        let mut meshes = vec![];
        for game_object in &self.game_objects {
            let object = game_object.component.borrow();
            match object.get_game_object_type() {
                GameObjectType::EnginePrimitive(engine_primitive) => match engine_primitive {
                    EnginePrimitiveType::StaticMesh(mesh_handle) => {
                        meshes.push((mesh_handle, object.get_transform()));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        meshes
    }
}
