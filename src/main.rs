use app::App;
use assets::AssetManager;
use core::EngineContext;
use game_object::primitives::static_mesh::StaticMesh;
use game_object::traits::GameObjectDefaults;
use input::{AxisAction, AxisBinding, InputAction, InputBinding, InputManager, KeyCode};
use material::material_manager::MaterialManager;
use material::{MaterialColorParameter, MaterialParameter, PbrMaterial};
use nalgebra_glm::{vec3, vec4};
use rendering_backend::backend_impl::resource_manager::ResourceManager;
use scene::scene::SceneManager;

fn main() {
    let mut engine_context = EngineContext::new();
    let scene_manager = SceneManager::new();
    let asset_manager = AssetManager::default();
    let resource_manager = ResourceManager::new();
    let material_manager = MaterialManager::new();

    // Setup InputManager with default bindings
    let mut input_manager = InputManager::new();

    // Bind movement actions
    input_manager.bind_action("move_forward", vec![InputBinding::Key(KeyCode::W)]);
    input_manager.bind_action("move_backward", vec![InputBinding::Key(KeyCode::S)]);
    input_manager.bind_action("move_left", vec![InputBinding::Key(KeyCode::A)]);
    input_manager.bind_action("move_right", vec![InputBinding::Key(KeyCode::D)]);

    // Bind axes for movement
    input_manager.bind_axis(
        AxisAction::HORIZONTAL,
        AxisBinding::Composite {
            positive: InputAction::from("move_right"),
            negative: InputAction::from("move_left"),
        },
    );
    input_manager.bind_axis(
        AxisAction::VERTICAL,
        AxisBinding::Composite {
            positive: InputAction::from("move_forward"),
            negative: InputAction::from("move_backward"),
        },
    );

    engine_context.register_manager(resource_manager);
    engine_context.register_manager(scene_manager);
    engine_context.register_manager(asset_manager);
    engine_context.register_manager(material_manager);
    engine_context.register_manager(input_manager);

    let app = App::new(engine_context);
    let mesh_handle = app
        .get_from_context::<AssetManager>()
        .get_mesh(".\\resources\\models\\test.obj");

    let material = PbrMaterial {
        base_color: MaterialColorParameter::Constant(vec4(255.0, 0.0, 0.0, 0.0)),
        normal: MaterialColorParameter::Constant(vec4(0.0, 0.0, 0.0, 0.0)),
        ambient_occlusion: MaterialParameter::Constant(0.0),
        roughness: MaterialParameter::Constant(0.0),
        specular: MaterialParameter::Constant(0.0),
        metallic: MaterialParameter::Constant(0.0),
    };

    let material = app
        .get_from_context::<MaterialManager>()
        .add_material_instance(material);

    let image = app
        .get_from_context::<AssetManager>()
        .get_image(".\\resources\\textures\\texture.png");

    let material2 = PbrMaterial {
        base_color: MaterialColorParameter::Handle(image.unwrap()),
        normal: MaterialColorParameter::Constant(vec4(0.0, 0.0, 0.0, 0.0)),
        ambient_occlusion: MaterialParameter::Constant(0.0),
        roughness: MaterialParameter::Constant(0.0),
        specular: MaterialParameter::Constant(0.0),
        metallic: MaterialParameter::Constant(0.0),
    };

    let material2 = app
        .get_from_context::<MaterialManager>()
        .add_material_instance(material2);

    let static_mesh2 = StaticMesh::new(mesh_handle.unwrap())
        .with_location(vec3(5.0, 1.0, 5.0))
        .with_material(material);
    let static_mesh3 = StaticMesh::new(mesh_handle.unwrap())
        .with_location(vec3(5.0, 10.0, 5.0))
        .with_scale(vec3(0.3, 0.3, 0.3))
        .with_material(material2);

    app.get_from_context::<SceneManager>()
        .register_game_object(static_mesh2);

    app.get_from_context::<SceneManager>()
        .register_game_object(static_mesh3);

    app.run();
}
