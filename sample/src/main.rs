use app::App;
use core::components::{
    CameraComponent, CameraControllerComponent, DirectionalLightComponent, MaterialComponent,
    MeshComponent, TransformComponent,
};
use core::system::{Context, System};
use core::types::transform::Transform;
use ecs::command_buffer::Commands;
use ecs::query::Query;
use input::{AnalogSource, AxisAction, AxisBinding, InputAction, InputBinding, KeyCode};
use nalgebra_glm::vec3;

#[allow(dead_code)]
mod assets {
    #[allow(unused_imports)]
    use common::{guid, Guid};
    include!(concat!(env!("OUT_DIR"), "/assets.rs"));
}

fn spawn_cube_system(
    _query: Query<(&mut MeshComponent, &mut MaterialComponent)>,
    ctx: &mut Context,
    commands: &mut Commands,
) {
    if !ctx.input.is_action_just_pressed("spawn_cube") {
        return;
    }

    let mesh_handle = ctx.load_mesh(assets::CUBE_OBJ);
    let material_handle = ctx.load_material(assets::BRICK_EMAT);

    let x = rand::random::<f32>() * 20.0 - 10.0;
    let y = rand::random::<f32>() * 20.0 - 10.0;
    let z = rand::random::<f32>() * 20.0 - 10.0;

    commands.spawn_entity((
        TransformComponent(Transform::default().with_location(vec3(x, y, z))),
        MeshComponent { mesh_handle },
        MaterialComponent { material_handle },
    ));
}

fn main() {
    let mut app = App::with_project("sample/sample.eproj");

    {
        let ctx = app.engine_context_mut();

        ctx.input_mut()
            .bind_action("spawn_cube", vec![InputBinding::Key(KeyCode::Space)]);
        ctx.input_mut()
            .bind_action("move_forward", vec![InputBinding::Key(KeyCode::W)]);
        ctx.input_mut()
            .bind_action("move_backward", vec![InputBinding::Key(KeyCode::S)]);
        ctx.input_mut()
            .bind_action("move_left", vec![InputBinding::Key(KeyCode::A)]);
        ctx.input_mut()
            .bind_action("move_right", vec![InputBinding::Key(KeyCode::D)]);

        ctx.input_mut().bind_axis(
            AxisAction::HORIZONTAL,
            AxisBinding::Composite {
                positive: InputAction::from("move_right"),
                negative: InputAction::from("move_left"),
            },
        );
        ctx.input_mut().bind_axis(
            AxisAction::VERTICAL,
            AxisBinding::Composite {
                positive: InputAction::from("move_forward"),
                negative: InputAction::from("move_backward"),
            },
        );
        ctx.input_mut().bind_axis(
            AxisAction::MOUSE_X,
            AxisBinding::Analog {
                source: AnalogSource::MouseX,
                sensitivity: 5.0,
            },
        );
        ctx.input_mut().bind_axis(
            AxisAction::MOUSE_Y,
            AxisBinding::Analog {
                source: AnalogSource::MouseY,
                sensitivity: 5.0,
            },
        );

        let floor_mesh = ctx.load_mesh(assets::FLOOR_OBJ);
        let cube_mesh = ctx.load_mesh(assets::CUBE_OBJ);
        let mat = ctx.load_material(assets::BRICK_EMAT);
        let mat2 = ctx.load_material(assets::SCROLL_EMAT);

        let setup = ctx.world_setup();

        setup.world.create_entity((
            TransformComponent(Transform::default()),
            MeshComponent {
                mesh_handle: floor_mesh,
            },
            MaterialComponent {
                material_handle: mat,
            },
        ));

        let mut iteration = 0;
        for x in 0..2i32 {
            for y in 0..2i32 {
                for z in 0..2i32 {
                    let selected = if iteration % 2 == 0 { mat } else { mat2 };

                    setup.world.create_entity((
                        TransformComponent(Transform::default().with_location(vec3(
                            x as f32 * 3.0,
                            1.0 + y as f32 * 3.0,
                            z as f32 * 3.0,
                        ))),
                        MeshComponent {
                            mesh_handle: cube_mesh,
                        },
                        MaterialComponent {
                            material_handle: selected,
                        },
                    ));

                    iteration += 1;
                }
            }
        }

        setup.world.create_entity((
            TransformComponent(Transform::default()),
            CameraComponent {
                near_clip: 0.1,
                far_clip: 1000.0,
                fov: 70.0,
                active: true,
            },
            CameraControllerComponent::new(50.0),
        ));

        setup.world.create_entity((
            TransformComponent(
                Transform::default().with_rotation(nalgebra_glm::normalize(&vec3(0.5, 1.0, 0.5))),
            ),
            DirectionalLightComponent {
                ambient_color: vec3(1.0, 1.0, 1.0),
                color: vec3(1.0, 1.0, 1.0),
                ambient_intensity: 0.1,
                intensity: 1.0,
            },
        ));

        ctx.register_system(Box::new(System::new(core::systems::basic_camera_system)));
        ctx.register_system(Box::new(System::new(spawn_cube_system)));
    }

    app.run();
}
