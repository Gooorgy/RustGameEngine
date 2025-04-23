use assets::{Asset, AssetManager, MeshAsset};

use core::EngineContext;
use macros::component;
use material::Material;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use vulkan_backend::scene::Transform;

// TODO: Define
#[derive(Eq, Hash, PartialEq)]
pub enum LifeCycleHook {
    Init,
    Update,
}

pub struct ComponentRegistration {
    pub component: Rc<RefCell<dyn SceneComponent>>,
    pub hooks: HashMap<LifeCycleHook, Hook>,
}

type Hook = fn(&mut EngineContext);

impl ComponentRegistration {
    pub fn new(component: impl SceneComponent + 'static) -> Self {
        Self {
            component: Rc::new(RefCell::new(component)),
            hooks: HashMap::new(),
        }
    }

    pub fn register_hook(&mut self, life_cycle: LifeCycleHook, hook_fn: Hook) {
        self.hooks.insert(life_cycle, hook_fn);
    }
}

pub trait SceneComponent: Any + Component {
    fn setup(
        &mut self,
        engine_context: &mut EngineContext,
    );

    fn as_any(&self) -> &dyn Any;
}

#[component]
pub struct StaticMesh {
    mesh_path: String,
    mesh_asset: Option<Rc<Asset<MeshAsset>>>,
    material: Option<Rc<Material>>,
}

impl SceneComponent for StaticMesh {
    fn setup(
        &mut self,
        engine_context: &mut EngineContext,
    ) {
        let asset_manager = engine_context.get::<AssetManager>().unwrap();
        self.init_assets(asset_manager);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl StaticMesh {
    pub fn init_assets(&mut self, asset_manager: &mut AssetManager) {
        self.mesh_asset = match asset_manager.get_mesh(&self.mesh_path) {
            Some(mesh) => Some(mesh),
            _ => panic!(),
        };

        if let Some(material) = self.material.as_ref() {}

        let am = AssetManager::default();
        let x = Rc::new(RefCell::new(am));
    }
}

impl StaticMesh {
    pub fn new(mesh_path: String) -> Self {
        Self {
            mesh_path,
            mesh_asset: None,
            transform: Transform::default(),
            material: None,
        }
    }

    pub fn with_material(&mut self, material: Material) {
        self.material = Some(Rc::new(material));
    }

    pub fn get_mesh(&self) -> Rc<Asset<MeshAsset>> {
        let x = match &self.mesh_asset {
            Some(mesh) => mesh,
            _ => panic!("Todo?"),
        };

        x.clone()
    }
}

pub trait Component {
    fn get_transform(&self) -> &Transform {
        todo!()
    }
}
