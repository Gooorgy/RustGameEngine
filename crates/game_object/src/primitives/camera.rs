use core::types::transform::Transform;
use macros::primitive_game_object;
use nalgebra_glm::{
    clamp_scalar, identity, rotate_x, rotate_y, rotate_z, translate, vec1, vec3, Mat4, Vec3, Vec4,
};

#[primitive_game_object]
pub struct Camera {
    pub velocity: Vec3,

    pub near_clip: f32,
    pub far_clip: f32,

    pub fov: f32,
}

// impl HasGameObjectType for Camera {
//     fn get_game_object_type(&self) -> GameObjectType {
//         GameObjectType::EnginePrimitive(EnginePrimitiveType::Camera(CameraData {
//             view: self.get_view_matrix(),
//             projection: Self::get_projection_matrix(),
//             far_clip: self.far_clip,
//             near_clip: self.near_clip,
//             fov: self.fov,
//         }))
//     }
// }

impl Camera {
    pub fn new() -> Self {
        Self {
            velocity: Vec3::new(0.0, 0.0, 0.0),
            transform: Transform::default(),
            fov: 70.0,
            near_clip: 0.1,
            far_clip: 1000.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        //let camera_rotation = self.get_rotation_matrix();

        let rot_x = rotate_x(&identity(), self.transform.rotation.x);
        let rot_y = rotate_y(&identity(), self.transform.rotation.y);
        let rot_z = rotate_z(&identity(), self.transform.rotation.z);
        let rotation = rot_z * rot_y * rot_x;

        let ff = rotation
            * (Vec4::new(self.velocity.x, self.velocity.y, self.velocity.z, 0.0)
                * 50.0
                * delta_time);

        let x = vec3(ff.x, ff.y, ff.z);
        self.transform.location += x;
    }

    // pub fn process_keyboard_event(&mut self, key_event: RawKeyEvent) {
    //     if key_event.state == ElementState::Pressed {
    //         if key_event.physical_key == KeyCode::KeyW {
    //             self.velocity.z = -1.0
    //         }
    //         if key_event.physical_key == KeyCode::KeyS {
    //             self.velocity.z = 1.0
    //         }
    //         if key_event.physical_key == KeyCode::KeyA {
    //             self.velocity.x = -1.0
    //         }
    //         if key_event.physical_key == KeyCode::KeyD {
    //             self.velocity.x = 1.0
    //         }
    //     }
    //     if key_event.state == ElementState::Released {
    //         if key_event.physical_key == KeyCode::KeyW {
    //             self.velocity.z = 0.0
    //         }
    //         if key_event.physical_key == KeyCode::KeyS {
    //             self.velocity.z = 0.0
    //         }
    //         if key_event.physical_key == KeyCode::KeyA {
    //             self.velocity.x = 0.0
    //         }
    //         if key_event.physical_key == KeyCode::KeyD {
    //             self.velocity.x = 0.0
    //         }
    //     }
    // }

    pub fn process_cursor_moved(&mut self, mouse_x: f32, mouse_y: f32) {
        self.transform.rotation.y -= mouse_x / 200.0;
        self.transform.rotation.x -= mouse_y / 200.0;

        let x = vec1(-89.0);

        self.transform.rotation.x = clamp_scalar(
            self.transform.rotation.x,
            -89.0_f32.to_radians(),
            89.0_f32.to_radians(),
        );
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let translation = translate(&identity(), &self.transform.location);

        let rot_x = rotate_x(&identity(), self.transform.rotation.x);
        let rot_y = rotate_y(&identity(), self.transform.rotation.y);
        let rot_z = rotate_z(&identity(), self.transform.rotation.z);
        let rotation = rot_z * rot_y * rot_x;

        let mat = translation * rotation;
        nalgebra_glm::inverse(&mat)
    }

    pub fn get_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        let mut projection = nalgebra_glm::perspective(
            aspect_ratio,
            self.fov.to_radians(),
            self.near_clip,
            self.far_clip,
        );
        projection[(1, 1)] *= -1.0;

        projection
    }

    pub fn get_projection_matrix_with_splits(&self, near_clip: f32, far_clip: f32) -> Mat4 {
        let aspect_ratio = 800.0 / 600.0;

        let mut projection =
            nalgebra_glm::perspective(aspect_ratio, 70_f32.to_radians(), near_clip, far_clip);
        projection[(1, 1)] *= -1.0;

        projection
    }
}
