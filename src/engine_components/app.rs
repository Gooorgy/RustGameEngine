use crate::assets::asset_manager::{Asset, AssetManager, MeshAsset};
use crate::engine_components::scene::{SceneComponent, StaticMesh};
use crate::vulkan_render::render_objects::draw_objects::Mesh;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub struct App {
    components: Vec<ComponentRegistration>,
    asset_manager: AssetManager,
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
        for mut component in self.components.iter_mut() {
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

pub struct ComponentRegistration {
    component: Rc<RefCell<dyn SceneComponent>>,
    hooks: Vec<Box<dyn Fn(&mut App)>>,
}

impl ComponentRegistration {
    pub fn new(component: impl SceneComponent + 'static) -> Self {
        Self {
            component: Rc::new(RefCell::new(component)),
            hooks: Vec::new(),
        }
    }

    pub fn register_hook(&mut self, hook: String) {
        // Finalize the concept of added lifecycle hooks
        todo!()
    }
}
