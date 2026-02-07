use crate::query::{Query, QueryParameter};
use crate::world::World;
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::marker::PhantomData;
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
        self.get_manager::<T>().expect(&format!(
            "Manager '{}' not found in ManagerContext",
            std::any::type_name::<T>()
        ))
    }
}

pub struct System<T: 'static + QueryParameter> {
    func: fn(Query<'_, T>, &ManagerContext),
    _phantom: PhantomData<T>,
}

impl<T: 'static + QueryParameter> System<T> {
    pub fn new(func: fn(Query<'_, T>, &ManagerContext)) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }
}

impl<T> SystemFunction for System<T>
where
    T: for<'a> QueryParameter + 'static,
{
    fn run(&self, world: &mut World, ctx: &ManagerContext) {
        let mut query = Query {
            world,
            matches: vec![],
        };

        query.build_matches();
        (self.func)(query, ctx);
    }
}

pub trait SystemFunction {
    fn run(&self, world: &mut World, ctx: &ManagerContext);
}
