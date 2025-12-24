use crate::registration::GameObjectRegistration;
use assets::MeshHandle;
use game_object::primitives::types::EnginePrimitiveType;
use game_object::traits::{GameObject, GameObjectType};
use core::types::transform::Transform;
use material::material_manager::MaterialHandle;

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

    pub fn get_static_meshes(&mut self) -> Vec<SceneMeshData> {
        let mut meshes = vec![];
        for game_object in &self.game_objects {
            let object = game_object.component.borrow();
            match object.get_game_object_type() {
                GameObjectType::EnginePrimitive(engine_primitive) => match engine_primitive {
                    EnginePrimitiveType::StaticMesh(mesh_data) => {
                        let scene_mesh_data = SceneMeshData {
                            mesh_handle: mesh_data.mesh_handle,
                            material_handle: mesh_data.material_handle.expect("missing material handle"),
                            transform: object.get_transform(),
                        };
                        meshes.push(scene_mesh_data);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        meshes
    }
}

pub struct SceneMeshData {
    pub mesh_handle: MeshHandle,
    pub transform: Transform,
    pub material_handle: MaterialHandle
}
