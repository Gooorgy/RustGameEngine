use crate::components::{CameraComponent, CameraControllerComponent};
use crate::system::Context;
use crate::TransformComponent;
use ecs::query::Query;
use input::AxisAction;
use nalgebra_glm::{identity, rotate_x, rotate_y, vec3, Vec4};
use ecs::command_buffer::Commands;

pub fn basic_camera_system(
    mut query: Query<(
        &mut CameraComponent,
        &mut TransformComponent,
        &mut CameraControllerComponent,
    )>,
    context: &mut Context,
    _commands: &mut Commands,
) {
    for (camera, transform, controller) in &mut query.iter() {
        if camera.active {
            let delta = context.dt;
            let input = context.input;

            let mouse_x = input.get_axis(AxisAction::MOUSE_X);
            let mouse_y = input.get_axis(AxisAction::MOUSE_Y);

            controller.yaw -= mouse_x * delta;
            controller.pitch -= mouse_y * delta;

            controller.pitch = controller
                .pitch
                .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

            transform.rotation.x = controller.pitch;
            transform.rotation.y = controller.yaw;

            let movement_x = input.get_axis("horizontal");
            let movement_z = -input.get_axis("vertical");

            let rot_x = rotate_x(&identity(), controller.pitch);
            let rot_y = rotate_y(&identity(), controller.yaw);
            let rotation = rot_y * rot_x;

            let velocity =
                rotation * (Vec4::new(movement_x, 0.0, movement_z, 0.0) * controller.speed * delta);
            transform.location += vec3(velocity.x, velocity.y, velocity.z);
        }
    }
}