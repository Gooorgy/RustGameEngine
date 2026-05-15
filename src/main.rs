use app::App;
use core::components::{
    CameraComponent, CameraControllerComponent, DirectionalLightComponent, MaterialComponent,
    MeshComponent, TransformComponent,
};
use core::types::transform::Transform;
use core::system::System;
use input::{AnalogSource, AxisAction, AxisBinding, InputAction, InputBinding, KeyCode};
use nalgebra_glm::vec3;
use spatial::{ColliderComponent, Shape};

fn main() {
    let mut app = App::new();

    {
        let ctx = app.engine_context_mut();

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

        let (floor_mesh, cube_mesh, mat) = {
            let floor = ctx.asset_context().load_mesh("resources\\models\\floor.obj");
            let cube = ctx.asset_context().load_mesh("resources\\models\\cube.obj");
            let mat = ctx.load_material("resources\\materials\\brick.emat");
            (floor, cube, mat)
        };

        let setup = ctx.world_setup();

        let collider = setup.spatial.register_collider(Shape::Sphere { radius: 1.0 });
        let collider2 = setup.spatial.register_collider(Shape::Sphere { radius: 2.0 });
        let collider3 = setup.spatial.register_collider(Shape::Sphere { radius: 1.0 });

        setup.world.create_entity((
            TransformComponent(Transform::default()),
            MeshComponent { mesh_handle: floor_mesh },
            MaterialComponent { material_handle: mat },
        ));

        setup.world.create_entity((
            TransformComponent(
                Transform::default()
                    .with_location(vec3(0.0, 1.0, 0.0))
                    .with_scale(vec3(1.0, 1.0, 1.0)),
            ),
            MeshComponent { mesh_handle: cube_mesh },
            MaterialComponent { material_handle: mat },
            ColliderComponent { id: collider },
        ));

        setup.world.create_entity((
            TransformComponent(
                Transform::default()
                    .with_location(vec3(0.0, 1.0, 5.0))
                    .with_scale(vec3(2.0, 2.0, 2.0)),
            ),
            MeshComponent { mesh_handle: cube_mesh },
            MaterialComponent { material_handle: mat },
            ColliderComponent { id: collider2 },
        ));

        setup.world.create_entity((
            TransformComponent(
                Transform::default()
                    .with_location(vec3(8.0, 12.0, 10.0))
                    .with_scale(vec3(1.0, 1.0, 1.0)),
            ),
            MeshComponent { mesh_handle: cube_mesh },
            MaterialComponent { material_handle: mat },
            ColliderComponent { id: collider3 },
        ));

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
    }

    app.run();
}
