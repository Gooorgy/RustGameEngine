use assets::{Asset, AssetManager, MeshAsset};

use core::EngineContext;
use macros::component;
use material::Material;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
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
    fn setup(&mut self, engine_context: &EngineContext);

    fn as_any(&self) -> &dyn Any;
}

#[component]
pub struct StaticMesh {
    mesh_path: String,
    mesh_asset: Option<Rc<Asset<MeshAsset>>>,
    material: Option<Rc<Material>>,
}

impl SceneComponent for StaticMesh {
    fn setup(&mut self, engine_context: &EngineContext) {
        let mut binding = engine_context.expect_system_mut::<AssetManager>();
        let asset_manager = binding.deref_mut();
        self.init_assets(asset_manager);

        println!("Initializing assets...");
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
        println!("Setup Mesh")
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
