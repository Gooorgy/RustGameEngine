use crate::asset_context::AssetContext;
use crate::TransformComponent;
use assets::AssetManager;
use ecs::systems::ManagerContext;
use ecs::world::World;
use input::InputManager;
use material::material_manager::MaterialManager;
use project::{AssetRegistry, Project};
use spatial::{ColliderComponent, SpatialWorld};
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

/// Provides simultaneous mutable access to both worlds, avoiding split-borrow issues.
pub struct WorldSetup<'a> {
    pub world: &'a mut World,
    pub spatial: &'a mut SpatialWorld,
}

/// Central engine context. Owns the asset context, ECS world, spatial world, and input.
pub struct EngineContext {
    assets: AssetContext,
    input_manager: Rc<RefCell<InputManager>>,
    world: World,
    spatial_world: SpatialWorld,
}

impl EngineContext {
    pub fn new(project: Project, registry: AssetRegistry) -> EngineContext {
        Self {
            assets: AssetContext::new(project, registry),
            input_manager: Rc::new(RefCell::new(InputManager::new())),
            world: World::new(),
            spatial_world: SpatialWorld::new(),
        }
    }

    /// Returns the asset context for all asset loading and GUID lookups.
    pub fn asset_context(&mut self) -> &mut AssetContext {
        &mut self.assets
    }

    // ── Renderer-facing shims ──────────────────────────────────────────────
    // These expose the inner managers directly so `engine.rs` can pass them
    // to the renderer without going through `AssetContext`.

    pub fn assets(&self) -> RefMut<'_, AssetManager> {
        self.assets.raw_assets()
    }

    pub fn materials(&self) -> RefMut<'_, MaterialManager> {
        self.assets.raw_materials()
    }

    pub fn input(&self) -> RefMut<'_, InputManager> {
        self.input_manager.borrow_mut()
    }

    // ── World access ───────────────────────────────────────────────────────

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

    // ── Frame update ───────────────────────────────────────────────────────

    pub fn update(&mut self, delta_time: f32) {
        let managers: HashMap<TypeId, Box<dyn Any>> = [
            (
                TypeId::of::<AssetManager>(),
                Box::new(self.assets.asset_manager_rc()) as Box<dyn Any>,
            ),
            (
                TypeId::of::<MaterialManager>(),
                Box::new(self.assets.material_manager_rc()) as Box<dyn Any>,
            ),
            (
                TypeId::of::<InputManager>(),
                Box::new(self.input_manager.clone()) as Box<dyn Any>,
            ),
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