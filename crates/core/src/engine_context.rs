use crate::asset_context::AssetContext;
use crate::system::{Context, SystemFunction};
use crate::TransformComponent;
use assets::AssetStore;
use config::config::{WindowMode, WindowResolution};
use ecs::world::World;
use input::InputManager;
use material::material_manager::{MaterialHandle, MaterialManager};
use project::Guid;
use spatial::{ColliderComponent, SpatialWorld};
use std::collections::HashMap;
use std::path::PathBuf;

/// Provides simultaneous mutable access to both worlds, avoiding split-borrow issues.
pub struct WorldSetup<'a> {
    pub world: &'a mut World,
    pub spatial: &'a mut SpatialWorld,
}

pub struct EngineConfig {
    pub name: String,
    pub content_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub window_resolution: WindowResolution,
    pub window_mode: WindowMode,
}

/// Central engine context. Owns engine config, asset context, ECS world, spatial world, input, and materials.
pub struct EngineContext {
    pub config: EngineConfig,
    assets: AssetContext,
    material_manager: MaterialManager,
    input_manager: InputManager,
    world: World,
    spatial_world: SpatialWorld,
    systems: Vec<Box<dyn SystemFunction>>,
}

impl EngineContext {
    pub fn new(config: EngineConfig, assets: AssetContext) -> EngineContext {
        Self {
            config,
            assets,
            material_manager: MaterialManager::new(),
            input_manager: InputManager::new(),
            world: World::new(),
            spatial_world: SpatialWorld::new(),
            systems: Vec::new(),
        }
    }

    // ── Asset loading ──────────────────────────────────────────────────────

    pub fn load_mesh(&mut self, guid: Guid) -> common::MeshHandle {
        self.assets.load_mesh(guid)
    }

    pub fn load_material(&mut self, guid: Guid) -> MaterialHandle {
        let assets = &mut self.assets;
        self.material_manager.get_or_insert(guid, || assets.build_material(guid))
    }

    // ── Renderer-facing accessors ──────────────────────────────────────────

    pub fn shader_cache_dir(&self) -> PathBuf {
        self.config.cache_dir.join("shaders")
    }

    pub fn asset_store(&self) -> &AssetStore {
        self.assets.store()
    }

    pub fn materials(&self) -> &MaterialManager {
        &self.material_manager
    }

    pub fn materials_mut(&mut self) -> &mut MaterialManager {
        &mut self.material_manager
    }

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