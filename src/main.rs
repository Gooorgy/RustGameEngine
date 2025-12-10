use app::App;
use assets::AssetManager;
use core::EngineContext;
use nalgebra_glm::vec3;
use rendering_backend::backend_impl::resource_manager::ResourceManager;
use rendering_backend::transform::Transform;
use scene::{SceneManager, StaticMesh};

fn main() {
    let mut engine_context = EngineContext::new();
    let scene_manager = SceneManager::new();
    let asset_manager = AssetManager::default();
    let resource_manager = ResourceManager::new();

    engine_context.register_system(resource_manager);
    engine_context.register_system(scene_manager);
    engine_context.register_system(asset_manager);

    let app = App::new(engine_context);

    let static_mesh = StaticMesh::new(String::from(".\\resources\\models\\test.obj"))
        .with_transform(Transform {
            position: vec3(1.0, 1.0, 1.0),
            ..Transform::default()
        });

    let static_mesh2 = StaticMesh::new(String::from(".\\resources\\models\\test.obj"));

    app.get_from_context::<SceneManager>()
        .register_component(static_mesh);

    app.get_from_context::<SceneManager>()
        .register_component(static_mesh2);

    app.run();
}
