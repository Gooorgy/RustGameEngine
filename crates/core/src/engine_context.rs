use ecs::world::World;
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

/// Context that holds and manages different managers.
///
/// Managers can be registered and retrieved in a type-safe way.
pub struct EngineContext {
    managers: HashMap<TypeId, Box<dyn Any>>,
    world: World,
}

impl EngineContext {
    pub fn new() -> EngineContext {
        Self {
            managers: HashMap::new(),
            world: World::new(),
        }
    }

    pub fn get_world(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn update(&mut self, delta_time: f32) {
        let ctx = ecs::systems::ManagerContext::new(&self.managers, delta_time);
        self.world.update(&ctx);
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
