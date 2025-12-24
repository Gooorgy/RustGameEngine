use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

/// Context that holds and manages different systems.
///
/// Systems can be registered and retrieved in a type-safe way.
pub struct EngineContext {
    systems: HashMap<TypeId, Box<dyn Any>>,
}

impl EngineContext {
    pub fn new() -> EngineContext {
        Self {
            systems: HashMap::new(),
        }
    }

    /// Inserts a system into the `EngineContext`.
    ///
    /// # Example
    ///
    /// ```
    /// use core::engine_context::EngineContext;
    ///
    /// struct MySystem;
    /// let mut context = EngineContext::new();
    /// context.register_system(MySystem);
    /// ```
    pub fn register_system<T: 'static>(&mut self, system: T) {
        self.systems
            .insert(system.type_id(), Box::new(Rc::new(RefCell::new(system))));
    }

    /// Retrieves a mutable reference to a system.
    ///
    /// # Panics
    ///
    /// Panics if the system has not been registered.
    ///
    /// # Example
    ///
    /// ```
    /// use core::EngineContext;
    ///
    /// struct MySystem {
    ///     value: i32,
    /// }
    ///
    /// let mut context = EngineContext::new();
    /// context.register_system(MySystem { value: 42 });
    ///
    /// let mut my_system = context.expect_system_mut::<MySystem>();
    /// my_system.value += 1;
    /// assert_eq!(my_system.value, 43);
    /// ```
    pub fn expect_system_mut<T: 'static>(&'_ self) -> RefMut<'_, T> {
        // Ensure the reference stays within the context
        let system = self
            .systems
            .get(&TypeId::of::<T>())
            .and_then(|system| system.downcast_ref::<Rc<RefCell<T>>>())
            .expect(&format!(
                "System '{}' not found in EngineContext",
                std::any::type_name::<T>()
            ))
            .borrow_mut();

        system
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestSystem {
        value: i32,
    }

    #[test]
    fn test_register_and_retrieve_system() {
        let mut context = EngineContext::new();
        context.register_system(TestSystem { value: 10 });

        let mut system = context.expect_system_mut::<TestSystem>();
        assert_eq!(system.value, 10);

        // Modify the system
        system.value = 42;
    }

    #[test]
    #[should_panic(expected = "MissingSystem' not found in EngineContext")]
    fn test_retrieve_nonexistent_system_panics() {
        #[derive(Debug)]
        struct MissingSystem;

        let context = EngineContext::new();
        let _ = context.expect_system_mut::<MissingSystem>();
    }

    #[test]
    fn test_multiple_systems() {
        #[derive(Debug, PartialEq)]
        struct AnotherSystem {
            name: &'static str,
        }

        let mut context = EngineContext::new();
        context.register_system(TestSystem { value: 7 });
        context.register_system(AnotherSystem { name: "test" });

        let mut test_system = context.expect_system_mut::<TestSystem>();
        let mut another_system = context.expect_system_mut::<AnotherSystem>();

        assert_eq!(test_system.value, 7);
        assert_eq!(another_system.name, "test");

        test_system.value = 99;
        another_system.name = "changed";

        assert_eq!(test_system.value, 99);
        assert_eq!(another_system.name, "changed");
    }
}
