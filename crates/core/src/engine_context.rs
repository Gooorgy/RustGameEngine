use std::any::{Any, TypeId};
use std::collections::HashMap;

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
        self.systems.insert(system.type_id(), Box::new(system));
    }

    pub fn get<T: 'static>(&mut self) -> Option<&mut T> {
        self.systems.get_mut(&TypeId::of::<T>())?.downcast_mut::<T>()
    }
}
