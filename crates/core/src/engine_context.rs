use crate::TransformComponent;
use assets::AssetManager;
use ecs::world::World;
use input::InputManager;
use material::material_manager::MaterialManager;
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use spatial::{ColliderComponent, SpatialWorld};

/// Provides simultaneous mutable access to both worlds, avoiding split-borrow issues.
pub struct WorldSetup<'a> {
    pub world: &'a mut World,
    pub spatial: &'a mut SpatialWorld,
}

/// Context that holds and manages different managers.
///
/// Managers can be registered and retrieved in a type-safe way.
pub struct EngineContext {
    managers: HashMap<TypeId, Box<dyn Any>>,
    world: World,
    spatial_world: SpatialWorld,
}

impl EngineContext {
    pub fn new() -> EngineContext {
        let mut ctx = Self {
            managers: HashMap::new(),
            world: World::new(),
            spatial_world: SpatialWorld::new(),
        };
        ctx.register_manager(AssetManager::default());
        ctx.register_manager(MaterialManager::new());
        ctx.register_manager(InputManager::new());
        ctx
    }

    pub fn assets(&self) -> RefMut<AssetManager> {
        self.expect_manager_mut::<AssetManager>()
    }

    pub fn materials(&self) -> RefMut<MaterialManager> {
        self.expect_manager_mut::<MaterialManager>()
    }

    pub fn input(&self) -> RefMut<InputManager> {
        self.expect_manager_mut::<InputManager>()
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
        let ctx = ecs::systems::ManagerContext::new(&self.managers, delta_time);
        self.world.update(&ctx);
        self.sync_spatial();
    }

    fn sync_spatial(&mut self) {
        self.spatial_world.clear_tree();
        let updates = {
            let mut query = self.world
                .query::<(&mut TransformComponent, &mut ColliderComponent)>();
            query.iter().map(|(t, c)| (c.id, t.location)).collect::<Vec<_>>()
        };
        for (id, center) in updates {
            self.spatial_world.insert_collider(id, center);
        }
    }

    /// Returns a reference to the managers HashMap for creating ManagerContext
    pub fn managers(&self) -> &HashMap<TypeId, Box<dyn Any>> {
        &self.managers
    }

    /// Inserts a manager into the `EngineContext`.
    ///
    /// # Example
    ///
    /// ```
    /// use core::engine_context::EngineContext;
    ///
    /// struct MyManager;
    /// let mut context = EngineContext::new();
    /// context.register_manager(MyManager);
    /// ```
    pub fn register_manager<T: 'static>(&mut self, manager: T) {
        self.managers
            .insert(manager.type_id(), Box::new(Rc::new(RefCell::new(manager))));
    }

    /// Retrieves a mutable reference to a manager.
    ///
    /// # Panics
    ///
    /// Panics if the manager has not been registered.
    ///
    /// # Example
    ///
    /// ```
    /// use core::EngineContext;
    ///
    /// struct MyManager {
    ///     value: i32,
    /// }
    ///
    /// let mut context = EngineContext::new();
    /// context.register_manager(MyManager { value: 42 });
    ///
    /// let mut my_manager = context.expect_manager_mut::<MyManager>();
    /// my_manager.value += 1;
    /// assert_eq!(my_manager.value, 43);
    /// ```
    pub fn expect_manager_mut<T: 'static>(&'_ self) -> RefMut<'_, T> {
        let manager = self
            .managers
            .get(&TypeId::of::<T>())
            .and_then(|manager| manager.downcast_ref::<Rc<RefCell<T>>>())
            .expect(&format!(
                "Manager '{}' not found in EngineContext",
                std::any::type_name::<T>()
            ))
            .borrow_mut();

        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestManager {
        value: i32,
    }

    #[test]
    fn test_register_and_retrieve_manager() {
        let mut context = EngineContext::new();
        context.register_manager(TestManager { value: 10 });

        let mut manager = context.expect_manager_mut::<TestManager>();
        assert_eq!(manager.value, 10);

        // Modify the manager
        manager.value = 42;
    }

    #[test]
    #[should_panic(expected = "MissingManager' not found in EngineContext")]
    fn test_retrieve_nonexistent_manager_panics() {
        #[derive(Debug)]
        struct MissingManager;

        let context = EngineContext::new();
        let _ = context.expect_manager_mut::<MissingManager>();
    }

    #[test]
    fn test_multiple_managers() {
        #[derive(Debug, PartialEq)]
        struct AnotherManager {
            name: &'static str,
        }

        let mut context = EngineContext::new();
        context.register_manager(TestManager { value: 7 });
        context.register_manager(AnotherManager { name: "test" });

        let mut test_manager = context.expect_manager_mut::<TestManager>();
        let mut another_manager = context.expect_manager_mut::<AnotherManager>();

        assert_eq!(test_manager.value, 7);
        assert_eq!(another_manager.name, "test");

        test_manager.value = 99;
        another_manager.name = "changed";

        assert_eq!(test_manager.value, 99);
        assert_eq!(another_manager.name, "changed");
    }
}
