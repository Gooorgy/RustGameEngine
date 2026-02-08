use ecs::component::Component;

#[derive(Clone, Debug, Component)]
pub struct CameraComponent {
    pub near_clip: f32,
    pub far_clip: f32,
    pub fov: f32,
    pub active: bool,
}

#[derive(Component, Debug, Clone)]
pub struct CameraControllerComponent {
    pub speed: f32,       // Movement speed
    pub sensitivity: f32, // Mouse sensitivity
    pub yaw: f32,         // Horizontal rotation (around Y axis)
    pub pitch: f32,       // Vertical rotation (around X axis)
}

impl CameraControllerComponent {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}
