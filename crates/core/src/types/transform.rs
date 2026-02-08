use nalgebra_glm::{identity, rotate_x, rotate_y, rotate_z, scaling, translate, vec3, Mat4, Vec3};

#[derive(Clone, Debug, Copy)]
pub struct Transform {
    pub location: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            location: vec3(0.0, 0.0, 0.0),
            rotation: vec3(0.0, 0.0, 0.0),
            scale: vec3(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Transform {
        Self {
            location: position,
            rotation,
            scale,
        }
    }

    pub fn with_location(mut self, location: Vec3) -> Self {
        self.location = location;
        self
    }

    pub fn with_rotation(mut self, rotation: nalgebra_glm::Vec3) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: nalgebra_glm::Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        let translation = translate(&identity(), &self.location);

        let rot_x = rotate_x(&identity(), self.rotation.x);
        let rot_y = rotate_y(&identity(), self.rotation.y);
        let rot_z = rotate_z(&identity(), self.rotation.z);
        let rotation = rot_z * rot_y * rot_x;

        let scale = scaling(&self.scale);

        translation * rotation * scale
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let translation = translate(&identity(), &self.location);
        let rot_x = rotate_x(&identity(), self.rotation.x);
        let rot_y = rotate_y(&identity(), self.rotation.y);
        let rot_z = rotate_z(&identity(), self.rotation.z);
        let rotation = rot_z * rot_y * rot_x;
        nalgebra_glm::inverse(&(translation * rotation))
    }
}
