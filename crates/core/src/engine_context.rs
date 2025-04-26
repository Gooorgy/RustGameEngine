use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

pub struct EngineContext {
    systems: HashMap<TypeId, Box<dyn Any>>,
}

impl EngineContext {
    pub fn new() -> EngineContext {
        Self {
            systems: HashMap::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, system: T) {
        self.systems
            .insert(system.type_id(), Box::new(Rc::new(RefCell::new(system))));
    }

    pub fn get_mut<T: 'static>(&self) -> Option<RefMut<T>> {
        // Ensure the reference stays within the context
        let x = self
            .systems
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref::<Rc<RefCell<T>>>()
            .unwrap();

        Some(x.borrow_mut())
    }
}
