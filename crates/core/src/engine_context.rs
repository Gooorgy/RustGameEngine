use crate::asset_context::AssetContext;
use crate::system::{Context, SystemFunction};
use crate::TransformComponent;
use assets::AssetStore;
use ecs::world::World;
use input::InputManager;
use material::material_manager::{MaterialHandle, MaterialManager};
use project::{AssetRegistry, Guid, Project};
use spatial::{ColliderComponent, SpatialWorld};
use std::collections::HashMap;
use std::path::Path;

/// Provides simultaneous mutable access to both worlds, avoiding split-borrow issues.
pub struct WorldSetup<'a> {
    pub world: &'a mut World,
    pub spatial: &'a mut SpatialWorld,
}

/// Central engine context. Owns the asset context, ECS world, spatial world, input, and materials.
pub struct EngineContext {
    assets: AssetContext,
    material_manager: MaterialManager,
    guid_to_material: HashMap<Guid, MaterialHandle>,
    material_to_guid: HashMap<MaterialHandle, Guid>,
    input_manager: InputManager,
    world: World,
    spatial_world: SpatialWorld,
    systems: Vec<Box<dyn SystemFunction>>,
}

impl EngineContext {
    pub fn new(project: Project, registry: AssetRegistry) -> EngineContext {
        Self {
            assets: AssetContext::new(project, registry),
            material_manager: MaterialManager::new(),
            guid_to_material: HashMap::new(),
            material_to_guid: HashMap::new(),
            input_manager: InputManager::new(),
            world: World::new(),
            spatial_world: SpatialWorld::new(),
            systems: Vec::new(),
        }
    }

    /// Returns the asset context for raw asset loading and GUID lookups.
    pub fn asset_context(&mut self) -> &mut AssetContext {
        &mut self.assets
    }

    // ── Asset loading ──────────────────────────────────────────────────────

    /// Development convenience: loads a material from a `.emat` source path,
    /// registers it with the material manager, and returns a stable handle.
    ///
    /// This exists because game code currently has no editor to assign materials
    /// by GUID. Once a level system exists, materials will be referenced by GUID
    /// directly in level files and this path-based API will no longer be needed.
    pub fn load_material<P: AsRef<Path>>(&mut self, path: P) -> MaterialHandle {
        let (mat, guid) = self.assets.build_material(path);
        let handle = self.material_manager.add_material_instance(mat);
        if let Some(guid) = guid {
            self.guid_to_material.insert(guid, handle);
            self.material_to_guid.insert(handle, guid);
        }
        handle
    }

    /// Returns the stable GUID for a material handle loaded via `load_material`.
    pub fn material_guid(&self, handle: MaterialHandle) -> Option<Guid> {
        self.material_to_guid.get(&handle).copied()
    }

    // ── Renderer-facing accessors ──────────────────────────────────────────

    pub fn asset_store(&self) -> &AssetStore {
        self.assets.store()
    }

    pub fn materials(&self) -> &MaterialManager {
        &self.material_manager
    }

    pub fn materials_mut(&mut self) -> &mut MaterialManager {
        &mut self.material_manager
    }

    /// Returns the asset store and material manager together from one borrow.
    /// Needed because Rust cannot split-borrow through method call boundaries,
    /// even when the fields are genuinely independent.
    pub fn render_resources_mut(&mut self) -> (&AssetStore, &mut MaterialManager) {
        (&self.assets.asset_store, &mut self.material_manager)
    }

    pub fn input(&self) -> &InputManager {
        &self.input_manager
    }

    pub fn input_mut(&mut self) -> &mut InputManager {
        &mut self.input_manager
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
        let systems = std::mem::take(&mut self.systems);
        let custom = HashMap::new();

        for system in &systems {
            let mut ctx = Context {
                dt: delta_time,
                assets: &mut self.assets,
                input: &self.input_manager,
                custom: &custom,
            };
            system.run(&mut self.world, &mut ctx);
        }

        self.systems = systems;
        self.sync_spatial();
    }

    pub fn register_system(&mut self, system: Box<dyn SystemFunction>) {
        self.systems.push(system);
    }

    fn sync_spatial(&mut self) {
        self.spatial_world.clear_tree();
        let updates = {
            let mut query = self
                .world
                .query::<(&mut TransformComponent, &mut ColliderComponent)>();
            query
                .iter()
                .map(|(t, c)| (c.id, t.location))
                .collect::<Vec<_>>()
        };
        for (id, center) in updates {
            self.spatial_world.insert_collider(id, center);
        }
    }
}