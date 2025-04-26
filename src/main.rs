use app::App;
use assets::AssetManager;
use core::EngineContext;
use scene::{SceneManager, StaticMesh};

fn main() {
    let mut engine_context = EngineContext::new();
    let scene_manager = SceneManager::new();
    let asset_manager = AssetManager::default();

    engine_context.insert(scene_manager);
    engine_context.insert(asset_manager);

    let mut app = App::new(engine_context);

    let static_mesh = StaticMesh::new(String::from(".\\resources\\models\\test.obj"));

    app.get_from_context::<SceneManager>()
        .unwrap()
        .register_component(static_mesh);

    app.run();
}
