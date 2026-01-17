use app::App;
use assets::AssetManager;
use core::EngineContext;
use ecs::component::component_storage::World;
use ecs::component::query::Query;
use ecs::component::Component;
use ecs::systems::{System, SystemFunction};
use game_object::primitives::static_mesh::StaticMesh;
use game_object::traits::GameObjectDefaults;
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

    engine_context.register_system(resource_manager);
    engine_context.register_system(scene_manager);
    engine_context.register_system(asset_manager);
    engine_context.register_system(material_manager);

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

    let mut world = World::new();
    let comp = TestComponent {
        test: String::from("Hallo"),
        data: 0,
    };

    let x = world.create_entity(comp);
    println!("Created entity: {:?}", x);
    let comp1 = TestComponent {
        test: String::from("Hallo2"),
        data: 0,
    };
    let x3 = world.create_entity(comp1);
        println!("Created entity: {:?}", x3);
    // let comp2 = TestComponent {
    //     test: String::from("Hallo"),
    //     data: 0,
    // };


    let comps = TestComponent {
        test: String::from("Hallo from other archetype"),
        data: 0,
    };

    let compsss = Test {
        data: 0,
    };
    
    
    let s = world.create_entity((comps, compsss));

    
    fn test(mut query: Query<&mut TestComponent>) {
        let len = query.iter().count();
        println!("len: {}", len);
        let c = query.iter();
        for cc in c {
            println!("{:?}", cc.test);
        }
    }

    world.register_system(Box::new(System::new(test)));
    world.update();

    //world.create_entity((comp2, comp1));

    app.run();
}

pub struct Test {
    data: u32,
}

pub struct TestComponent {
    data: usize,
    test: String,
}

impl Component for TestComponent {}

impl Component for Test {
    
}
