use std::any::{Any, TypeId};
use std::rc::Rc;
use crate::assets::asset_manager::{Asset, AssetManager, MeshAsset};
use crate::engine_components::app::ComponentRegistration;
use crate::vulkan_render::render_objects::draw_objects::Mesh;
use crate::vulkan_render::scene::Transform;

pub trait SceneComponent: Any {
    fn setup(&self, registration: &mut ComponentRegistration);

    fn init_assets(&mut self, asset_manager: &mut AssetManager) {
        // optional
    }

    fn as_any(&self) -> &dyn Any;
}

pub struct StaticMesh {
    mesh_path: String,
    mesh_asset: Option<Rc<Asset<MeshAsset>>>,
    transform: Transform,
}

impl SceneComponent for StaticMesh {
    fn setup(&self, registration: &mut ComponentRegistration) {}
    fn init_assets(&mut self, asset_manager: &mut AssetManager) {
        self.mesh_asset = match asset_manager.get_mesh(&self.mesh_path) {
         Some(mesh) => Some(mesh),
            _ => panic!()
        };
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}



impl StaticMesh {
    pub fn new(mesh_path: String) -> Self {
        Self {
            mesh_path,
            mesh_asset: None,
            transform: Transform::default(),
        }
    }

    pub fn get_mesh(&self) -> Rc<Asset<MeshAsset>> {
        let x = match &self.mesh_asset {
            Some(mesh) => mesh,
            _ => panic!("Todo?")
        };

        x.clone()
    }
}
