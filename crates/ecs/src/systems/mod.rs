use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

/// Context passed to ECS systems, providing access to managers and frame data.
pub struct ManagerContext<'a> {
    managers: &'a HashMap<TypeId, Box<dyn Any>>,
    pub delta_time: f32,
}

impl<'a> ManagerContext<'a> {
    pub fn new(managers: &'a HashMap<TypeId, Box<dyn Any>>, delta_time: f32) -> Self {
        Self {
            managers,
            delta_time,
        }
    }

    /// Retrieves a mutable reference to a manager.
    pub fn get_manager<T: 'static>(&self) -> Option<RefMut<'_, T>> {
        self.managers
            .get(&TypeId::of::<T>())
            .and_then(|manager| manager.downcast_ref::<Rc<RefCell<T>>>())
            .map(|rc| rc.borrow_mut())
    }

    /// Retrieves a mutable reference to a manager, panicking if not found.
    pub fn expect_manager<T: 'static>(&self) -> RefMut<'_, T> {
        self.get_manager::<T>().unwrap_or_else(|| {
            panic!(
                "Manager '{}' not found in ManagerContext",
                std::any::type_name::<T>()
            )
        })
    }
}