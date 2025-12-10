use assets::{AssetManager, MaterialHandle, MeshHandle, Resolve};

use core::EngineContext;
use macros::component;
use rendering_backend::transform::Transform;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

// TODO: Define
#[derive(Eq, Hash, PartialEq)]
pub enum LifeCycleHook {
    Init,
    Update,
}
type Hook = fn(&mut EngineContext);

pub struct ResolvedComponent<T: Resolve> {
    pub data: T,             // original unresolved
    pub handles: T::Handles, // resolved asset handles
}

impl<T: Resolve + Clone> ResolvedComponent<T> {
    pub fn new(data: T, assets: &mut AssetManager) -> Self {
        let handles = data.resolve(assets);
        Self { data, handles }
    }
}

pub struct ComponentRegistration {
    pub component: Rc<RefCell<dyn SceneComponent>>,
}

impl ComponentRegistration {
    pub fn new(component: impl SceneComponent + 'static) -> Self {
        Self {
            component: Rc::new(RefCell::new(component)),
        }
    }
}

pub trait SceneComponent: Any + Component {
    fn setup(&mut self, engine_context: &EngineContext);

    fn as_any(&self) -> &dyn Any;
}

pub struct StaticMeshHandles {
    pub mesh: MeshHandle,
    pub material: Option<MaterialHandle>,
}

#[component]
#[derive(Clone)]
pub struct StaticMesh {
    mesh_path: String,
    material_path: Option<String>,
}

impl SceneComponent for StaticMesh {
    fn setup(&mut self, engine_context: &EngineContext) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Resolve for StaticMesh {
    type Handles = StaticMeshHandles;

    fn resolve(&self, asset_manager: &mut AssetManager) -> Self::Handles {
        let mesh_handle = asset_manager
            .get_mesh(&self.mesh_path)
            .expect("Mesh not found")
            .id;
        let material_handle = None;
        StaticMeshHandles {
            mesh: mesh_handle,
            material: material_handle,
        }
    }
}

impl StaticMesh {
    pub fn new(mesh_path: String) -> Self {
        Self {
            mesh_path,
            transform: Transform::default(),
            material_path: None,
        }
    }
    
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_material(&mut self, material_path: String) {
        self.material_path = Some(material_path);
    }
}

pub trait Component {
    fn get_transform(&self) -> Transform {
        todo!()
    }
}
