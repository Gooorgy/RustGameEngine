use assets::MeshHandle;
use core::types::transform::Transform;
use core::{CameraComponent, MaterialComponent, MeshComponent, TransformComponent};
use ecs::world::World;
use material::material_manager::MaterialHandle;
use nalgebra_glm::Mat4;

/// A request to render a mesh with a specific transform and material.
#[derive(Clone)]
pub struct MeshRenderRequest {
    pub mesh_handle: MeshHandle,
    pub material_handle: MaterialHandle,
    pub transform: Transform,
}

pub struct CameraRenderData {
    pub view: Mat4,
    pub proj: Mat4,
}

/// Collects render data from the ECS World.
/// Designed to be extensible for future render types (lights, particles, etc.)
pub struct RenderDataCollector {
    pub mesh_requests: Vec<MeshRenderRequest>,
    pub camera: Option<CameraRenderData>,
}

impl RenderDataCollector {
    pub fn new() -> Self {
        Self {
            mesh_requests: Vec::new(),
            camera: None,
        }
    }

    /// Collects all render data from the World by querying for renderable entities.
    pub fn collect_from_world(&mut self, world: &mut World, aspect_ratio: f32) {
        self.mesh_requests.clear();
        self.camera = None;
        self.collect_meshes(world);
        self.collect_camera(world, aspect_ratio);
    }

    fn collect_meshes(&mut self, world: &mut World) {
        let mut query = world.query::<(
            &mut TransformComponent,
            &mut MeshComponent,
            &mut MaterialComponent,
        )>();

        for (transform, mesh, material) in query.iter() {
            self.mesh_requests.push(MeshRenderRequest {
                mesh_handle: mesh.mesh_handle,
                material_handle: material.material_handle,
                transform: transform.0.clone(),
            });
        }
    }

    fn collect_camera(&mut self, world: &mut World, aspect_ratio: f32) {
        let mut query = world.query::<(&mut TransformComponent, &mut CameraComponent)>();
        for (transform, camera) in query.iter() {
            if camera.active {
                let view = transform.0.get_view_matrix();
                let mut proj = nalgebra_glm::perspective(
                    aspect_ratio,
                    camera.fov.to_radians(),
                    camera.near_clip,
                    camera.far_clip,
                );
                proj[(1, 1)] *= -1.0; // Vulkan Y-flip
                self.camera = Some(CameraRenderData { view, proj });
                break;
            }
        }
    }
}

impl Default for RenderDataCollector {
    fn default() -> Self {
        Self::new()
    }
}
