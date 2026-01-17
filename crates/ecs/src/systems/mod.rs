use crate::component::component_storage::World;
use crate::component::query::{Query, QueryParameter};
use std::marker::PhantomData;

// System wrapper to handle the lifetime issues
pub struct System<T: 'static + QueryParameter> {
    func: fn(Query<'_, T>),
    _phantom: PhantomData<T>,
}

impl<T: 'static + QueryParameter> System<T> {
    pub fn new(func: fn(Query<'_, T>)) -> Self {
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
    fn run(&self, world: &mut World) {
        let mut query = Query {
            world,
            matches: vec![]
        };

        query.build_matches();
        (self.func)(query);
    }
}


pub trait SystemFunction {
    fn run(&self, world: &mut World);
}
