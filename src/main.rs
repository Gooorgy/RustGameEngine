use app::App;
use assets::AssetManager;
use core::EngineContext;
use game_object::primitives::static_mesh::StaticMesh;
use game_object::traits::GameObjectDefaults;
use nalgebra_glm::vec3;
use rendering_backend::backend_impl::resource_manager::ResourceManager;
use scene::scene::SceneManager;

fn main() {
    let mut engine_context = EngineContext::new();
    let scene_manager = SceneManager::new();
    let asset_manager = AssetManager::default();
    let resource_manager = ResourceManager::new();

    engine_context.register_system(resource_manager);
    engine_context.register_system(scene_manager);
    engine_context.register_system(asset_manager);

    let app = App::new(engine_context);
    let mesh_handle = app
        .get_from_context::<AssetManager>()
        .get_mesh(".\\resources\\models\\test.obj");
    let static_mesh = StaticMesh::new(mesh_handle.unwrap());

    app.get_from_context::<SceneManager>()
        .register_game_object(static_mesh);

    let static_mesh2 = StaticMesh::new(mesh_handle.unwrap()).with_location(vec3(5.0, 1.0, 5.0));

    let static_mesh3 = StaticMesh::new(mesh_handle.unwrap()).with_location(vec3(5.0, 10.0, 5.0)).with_scale(vec3(0.3, 0.3, 0.3));
    
    app.get_from_context::<SceneManager>()
        .register_game_object(static_mesh2);
    
    app.get_from_context::<SceneManager>().register_game_object(static_mesh3);

    app.run();
}
