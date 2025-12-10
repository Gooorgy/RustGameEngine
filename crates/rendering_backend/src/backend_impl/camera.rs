// use glm::{vec3, Mat4, Vec3, Vec4};
// use winit::event::{ElementState, RawKeyEvent};
// use winit::keyboard::KeyCode;
//
// pub struct Camera {
//     position: Vec3,
//     velocity: Vec3,
//
//     pitch: f32,
//     yaw: f32,
//
//     pub near_clip: f32,
//     pub far_clip: f32,
// }
//
// impl Camera {
//     pub fn new() -> Self {
//         Self {
//             position: Vec3::new(0.0, 0.0, 0.0),
//             velocity: Vec3::new(0.0, 0.0, 0.0),
//             pitch: 0.0,
//             yaw: 0.0,
//
//             near_clip: 0.1,
//             far_clip: 1000.0,
//         }
//     }
//
//     pub fn update(&mut self, delta_time: f32) {
//         let camera_rotation = self.get_rotation_matrix();
//
//         let ff = camera_rotation
//             * (Vec4::new(self.velocity.x, self.velocity.y, self.velocity.z, 0.0)
//                 * 50.0
//                 * delta_time);
//
//         let x = vec3(ff.x, ff.y, ff.z);
//         self.position += x;
//     }
//
//     pub fn process_keyboard_event(&mut self, key_event: RawKeyEvent) {
//         if key_event.state == ElementState::Pressed {
//             if key_event.physical_key == KeyCode::KeyW {
//                 self.velocity.z = -1.0
//             }
//             if key_event.physical_key == KeyCode::KeyS {
//                 self.velocity.z = 1.0
//             }
//             if key_event.physical_key == KeyCode::KeyA {
//                 self.velocity.x = -1.0
//             }
//             if key_event.physical_key == KeyCode::KeyD {
//                 self.velocity.x = 1.0
//             }
//         }
//         if key_event.state == ElementState::Released {
//             if key_event.physical_key == KeyCode::KeyW {
//                 self.velocity.z = 0.0
//             }
//             if key_event.physical_key == KeyCode::KeyS {
//                 self.velocity.z = 0.0
//             }
//             if key_event.physical_key == KeyCode::KeyA {
//                 self.velocity.x = 0.0
//             }
//             if key_event.physical_key == KeyCode::KeyD {
//                 self.velocity.x = 0.0
//             }
//         }
//     }
//
//     pub fn process_cursor_moved(&mut self, mouse_x: f32, mouse_y: f32) {
//         self.yaw += mouse_x / 200.0;
//         self.pitch -= mouse_y / 200.0;
//
//         if self.pitch > 1.5 {
//             self.pitch = 1.5;
//         }
//         if self.pitch < -1.5 {
//             self.pitch = -1.5;
//         }
//     }
//
//     pub fn get_view_matrix(&self) -> Mat4 {
//         glm::inverse(&self.get_transform())
//     }
//
//     pub fn get_transform(&self) -> Mat4 {
//         let camera_translation = glm::translate(&Mat4::identity(), &self.position);
//         let camera_rotation = self.get_rotation_matrix();
//
//         camera_translation * camera_rotation
//     }
//
//     pub fn get_rotation_matrix(&self) -> Mat4 {
//         let pitch_rotation = glm::quat_angle_axis(self.pitch, &Vec3::new(1.0, 0.0, 0.0));
//         let yaw_rotation = glm::quat_angle_axis(self.yaw, &Vec3::new(0.0, -1.0, 0.0));
//
//         glm::quat_to_mat4(&yaw_rotation) * glm::quat_to_mat4(&pitch_rotation)
//     }
//
//     pub fn get_projection_matrix(&self) -> Mat4 {
//         let aspect_ratio = 800.0 / 600.0;
//
//         let mut projection = glm::perspective(
//             aspect_ratio,
//             70_f32.to_radians(),
//             self.near_clip,
//             self.far_clip,
//         );
//         projection[(1, 1)] *= -1.0;
//
//         projection
//     }
//
//     pub fn get_projection_matrix_with_splits(&self, near_clip: f32, far_clip: f32) -> Mat4 {
//         let aspect_ratio = 800.0 / 600.0;
//
//         let mut projection =
//             glm::perspective(aspect_ratio, 70_f32.to_radians(), near_clip, far_clip);
//         projection[(1, 1)] *= -1.0;
//
//         projection
//     }
// }
