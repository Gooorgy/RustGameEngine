use crate::{Component, ComponentRegistration, ResolvedComponent, SceneComponent, StaticMesh};
use assets::{AssetManager, MeshHandle};
use core::EngineContext;
use rendering_backend::transform::Transform;

pub struct SceneManager {
    pub scene_components: Vec<ComponentRegistration>,
    pub mesh_components: Vec<ResolvedComponent<StaticMesh>>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            scene_components: Vec::new(),
            mesh_components: Vec::new(),
        }
    }

    pub fn register_component(
        &mut self,
        component: impl SceneComponent + 'static,
    ) -> &ComponentRegistration {
        let registration = ComponentRegistration::new(component);
        self.add_registration(registration)
    }

    fn add_registration(&mut self, registration: ComponentRegistration) -> &ComponentRegistration {
        self.scene_components.push(registration);

        println!("Adding component registration");
        &self.scene_components[self.scene_components.len() - 1]
    }

    pub fn init_scene(&mut self, engine_context: &EngineContext) {
        let mut manager = engine_context.expect_system_mut::<AssetManager>();
        for component in &self.scene_components {
            let x = component
                .component
                .borrow_mut()
                .as_any()
                .downcast_ref::<StaticMesh>()
                .unwrap()
                .clone();
            let resolved = ResolvedComponent::new(x, &mut manager);
            self.mesh_components.push(resolved);
        }
    }

    // pub fn prepare_scene_data(&self) -> HashMap<AssetId, Transform> {
    //     self.scene_components
    //         .iter()
    //         .filter_map(|component| {
    //             component
    //                 .component
    //                 .borrow()
    //                 .as_any()
    //                 .downcast_ref::<StaticMesh>()
    //                 .cloned()
    //         })
    //         .map(|mesh| {
    //             let asset_id = mesh.get_mesh().clone(); // Clone or Copy the data
    //             let transform = mesh.get_transform().to_owned(); // Ensure ownership
    //             (asset_id, transform)
    //         })
    //         .collect()
    // }

    // TODO: There are currently only Static meshes. once there are multiple sceneComponents this has to go...
    pub fn get_static_meshes(&mut self) -> Vec<(MeshHandle, Transform)> {
        let mut meshes = vec![];
        for componentRegistration in &self.mesh_components {
            let static_mesh = componentRegistration.handles.mesh;
            let transform = componentRegistration.data.get_transform();

            meshes.push((static_mesh, transform));
        }

        meshes
    }
}
