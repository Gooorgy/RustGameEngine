use crate::assets::asset_manager::AssetManager;
use crate::engine_components::system::{HasAssetManager, HasGameState, InitReq, System};
use crate::vulkan_render::render_objects::draw_objects::Mesh;
use crate::vulkan_render::scene::Transform;

pub struct StaticMeshComponent {
    pub name: String,
    pub mesh: Mesh,
    pub transform: Transform
}

impl StaticMeshComponent {
    pub fn new(name: String, mesh: Mesh, transform: Transform) -> Self {
        StaticMeshComponent { name, mesh, transform }
    }
}

impl StaticMeshComponent {
    pub fn update(){}
}

impl GameObject for StaticMeshComponent {
    fn init(system: impl InitReq) {
        system.asset_manager();
    }
}

pub trait GameObject {
    fn init(system: impl InitReq);
}

pub trait Actor : GameObject {
    fn update();
}

pub struct Base {}

impl GameObject for Base {
    fn init(system: impl InitReq) {
        todo!()
    }
}

impl Actor for Base {
    fn update() {
        todo!()
    }
}

pub type Test = Box<dyn HasGameState>;