use crate::TransformComponent;
use asset_pipeline::EmatFile;
use assets::AssetManager;
use ecs::systems::ManagerContext;
use ecs::world::World;
use input::InputManager;
use material::material_manager::{MaterialHandle, MaterialManager};
use project::{AssetRegistry, Project};
use spatial::{ColliderComponent, SpatialWorld};
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

/// Provides simultaneous mutable access to both worlds, avoiding split-borrow issues.
pub struct WorldSetup<'a> {
    pub world: &'a mut World,
    pub spatial: &'a mut SpatialWorld,
}

/// Central engine context. Owns the ECS world, spatial world, project, and core manager instances.
pub struct EngineContext {
    project: Project,
    registry: AssetRegistry,
    asset_manager: Rc<RefCell<AssetManager>>,
    material_manager: Rc<RefCell<MaterialManager>>,
    input_manager: Rc<RefCell<InputManager>>,
    world: World,
    spatial_world: SpatialWorld,
}

impl EngineContext {
    pub fn new(project: Project, registry: AssetRegistry) -> EngineContext {
        Self {
            project,
            registry,
            asset_manager: Rc::new(RefCell::new(AssetManager::default())),
            material_manager: Rc::new(RefCell::new(MaterialManager::new())),
            input_manager: Rc::new(RefCell::new(InputManager::new())),
            world: World::new(),
            spatial_world: SpatialWorld::new(),
        }
    }

    /// Loads a `.emat` file from `path`, resolves its asset references through
    /// the project registry, and registers the result with the material manager.
    pub fn load_material<P: AsRef<Path>>(&mut self, path: P) -> MaterialHandle {
        let path = path.as_ref();
        let mat = {
            let mut assets = self.assets();
            EmatFile::load(path)
                .and_then(|f| f.build_material(&self.project, &self.registry, &mut assets))
                .unwrap_or_else(|e| panic!("failed to load '{}': {}", path.display(), e))
        };
        self.materials().add_material_instance(mat)
    }

    pub fn assets(&self) -> RefMut<'_, AssetManager> {
        self.asset_manager.borrow_mut()
    }

    pub fn materials(&self) -> RefMut<'_, MaterialManager> {
        self.material_manager.borrow_mut()
    }

    pub fn input(&self) -> RefMut<'_, InputManager> {
        self.input_manager.borrow_mut()
    }

    pub fn world_setup(&mut self) -> WorldSetup<'_> {
        WorldSetup {
            world: &mut self.world,
            spatial: &mut self.spatial_world,
        }
    }

    pub fn get_world(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn get_spatial_world(&self) -> &SpatialWorld {
        &self.spatial_world
    }

    pub fn get_spatial_world_mut(&mut self) -> &mut SpatialWorld {
        &mut self.spatial_world
    }

    pub fn update(&mut self, delta_time: f32) {
        // Rc::clone is O(1) — just bumps reference counts.
        // The HashMap is an implementation detail hidden from callers.
        let managers: HashMap<TypeId, Box<dyn Any>> = [
            (TypeId::of::<AssetManager>(), Box::new(self.asset_manager.clone()) as Box<dyn Any>),
            (TypeId::of::<MaterialManager>(), Box::new(self.material_manager.clone()) as Box<dyn Any>),
            (TypeId::of::<InputManager>(), Box::new(self.input_manager.clone()) as Box<dyn Any>),
        ]
        .into_iter()
        .collect();

        let ctx = ManagerContext::new(&managers, delta_time);
        self.world.update(&ctx);
        self.sync_spatial();
    }

    fn sync_spatial(&mut self) {
        self.spatial_world.clear_tree();
        let updates = {
            let mut query = self
                .world
                .query::<(&mut TransformComponent, &mut ColliderComponent)>();
            query.iter().map(|(t, c)| (c.id, t.location)).collect::<Vec<_>>()
        };
        for (id, center) in updates {
            self.spatial_world.insert_collider(id, center);
        }
    }
}
