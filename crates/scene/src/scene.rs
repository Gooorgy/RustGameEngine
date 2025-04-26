use crate::{ComponentRegistration, SceneComponent, StaticMesh};
use assets::{Asset, AssetManager, MeshAsset};
use core::EngineContext;
use std::rc::Rc;

pub trait Setup {
    fn setup(&mut self, context: &mut SetupContext);
}

pub struct SetupContext<'a> {
    pub asset_manager: &'a mut AssetManager,
    pub component_registration: &'a mut ComponentRegistration,
}

pub struct SceneManager {
    pub scene_components: Vec<ComponentRegistration>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            scene_components: Vec::new(),
        }
    }

    pub fn register_component(
        &mut self,
        component: impl SceneComponent + 'static,
    ) -> &ComponentRegistration {
        let registration = ComponentRegistration::new(component);
        self.add_registration(registration)
    }

    fn add_registration(&mut self, registration: ComponentRegistration) -> &ComponentRegistration {
        self.scene_components.push(registration);

        println!("Adding component registration");
        &self.scene_components[self.scene_components.len() - 1]
    }

    pub fn init_scene(&mut self, engine_context: &EngineContext) {
        println!("Initializing scene... {0}", self.scene_components.len());

        for component in &self.scene_components {
            component.component.borrow_mut().setup(engine_context);
        }
    }

    // TODO: There are currently only Static meshes. once there are multiple sceneComponents this has to go...
    pub fn get_static_meshes(&mut self) -> Vec<Rc<Asset<MeshAsset>>> {
        let mut meshes = vec![];
        for component in &self.scene_components {
            let borrow = component.component.borrow();
            let static_mesh = match borrow.as_any().downcast_ref::<StaticMesh>() {
                Some(s) => s.get_mesh(),
                None => continue,
            };

            meshes.push(static_mesh);
        }

        meshes
    }
}
