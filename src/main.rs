use app::App;
use assets::AssetManager;
use core::components::{
    CameraComponent, CameraControllerComponent, DirectionalLightComponent, MaterialComponent,
    MeshComponent, TransformComponent,
};
use core::types::transform::Transform;
use core::EngineContext;
use ecs::systems::{System, SystemFunction};
use input::{
    AnalogSource, AxisAction, AxisBinding, InputAction, InputBinding, InputManager, KeyCode,
};
use material::material_manager::MaterialManager;
use material::{MaterialColorParameter, MaterialParameter, PbrMaterial};
use nalgebra_glm::{vec3, vec4};
use rendering_backend::backend_impl::resource_manager::ResourceManager;

fn main() {
    let mut engine_context = EngineContext::new();
    let asset_manager = AssetManager::default();
    let resource_manager = ResourceManager::new();
    let material_manager = MaterialManager::new();

    let mut input_manager = InputManager::new();

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

    input_manager.bind_axis(
        AxisAction::MOUSE_X,
        AxisBinding::Analog {
            source: AnalogSource::MouseX,
            sensitivity: 5.0,
        },
    );

    input_manager.bind_axis(
        AxisAction::MOUSE_Y,
        AxisBinding::Analog {
            source: AnalogSource::MouseY,
            sensitivity: 5.0,
        },
    );

    engine_context.register_manager(resource_manager);
    engine_context.register_manager(asset_manager);
    engine_context.register_manager(material_manager);
    engine_context.register_manager(input_manager);

    let mut app = App::new(engine_context);
    let floor_mesh = app
        .get_from_context::<AssetManager>()
        .get_mesh(".\\resources\\models\\floor.obj");

    let cube_mesh = app
        .get_from_context::<AssetManager>()
        .get_mesh(".\\resources\\models\\cube.obj");

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

    let world = app.get_world();

    world.create_entity((
        TransformComponent(Transform::default()),
        MeshComponent {
            mesh_handle: floor_mesh.unwrap(),
        },
        MaterialComponent {
            material_handle: material,
        },
    ));

    world.create_entity((
        TransformComponent(
            Transform::default()
                .with_location(vec3(0.0, 1.0, 0.0))
                .with_scale(vec3(1.0, 1.0, 1.0)),
        ),
        MeshComponent {
            mesh_handle: cube_mesh.unwrap(),
        },
        MaterialComponent {
            material_handle: material2,
        },
    ));

    world.create_entity((
        TransformComponent(Transform::default()),
        CameraComponent {
            near_clip: 0.1,
            far_clip: 1000.0,
            fov: 70.0,
            active: true,
        },
        CameraControllerComponent::new(50.0),
    ));

    world.create_entity((
        TransformComponent(Transform::default().with_rotation(
            nalgebra_glm::normalize(&vec3(0.5, 1.0, 0.5))
        )),
        DirectionalLightComponent {
            ambient_color: vec3(1.0, 1.0, 1.0),
            color: vec3(1.0, 1.0, 1.0),
            ambient_intensity: 0.3,
            intensity: 1.0,
        },
    ));

    world.register_system(Box::new(System::new(core::systems::basic_camera_system)));
    app.run();
}
