use assets::{Asset, AssetManager, MeshAsset};
use scene::{ComponentRegistration, SceneComponent, StaticMesh};
use std::rc::Rc;

pub struct App {
    components: Vec<ComponentRegistration>,
    pub asset_manager: AssetManager,
}

impl App {
    pub fn new() -> App {
        Self {
            components: Vec::new(),
            asset_manager: AssetManager::default(),
        }
    }
    pub fn register_component(
        &mut self,
        component: impl SceneComponent + 'static,
    ) -> &ComponentRegistration {
        let registration = ComponentRegistration::new(component);
        self.add_registration(registration)
    }

    pub fn init(&mut self) {
        for component in self.components.iter_mut() {
            component
                .component
                .borrow_mut()
                .init_assets(&mut self.asset_manager);
        }
    }

    fn add_registration(&mut self, registration: ComponentRegistration) -> &ComponentRegistration {
        self.components.push(registration);
        &self.components[self.components.len() - 1]
    }

    pub fn get_static_meshes(&self) -> Vec<Rc<Asset<MeshAsset>>> {
        let mut meshes = vec![];
        for component in &self.components {
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
