use crate::asset_context::AssetContext;
use ecs::query::{Query, QueryParameter};
use ecs::world::World;
use input::InputManager;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct Context<'a> {
    pub dt: f32,
    pub assets: &'a mut AssetContext,
    pub input: &'a InputManager,
    pub custom: &'a HashMap<TypeId, Rc<RefCell<dyn Any>>>,
}

pub struct System<T: 'static + QueryParameter> {
    func: fn(Query<'_, T>, &Context),
    _phantom: PhantomData<T>,
}

impl<T: 'static + QueryParameter> System<T> {
    pub fn new(func: fn(Query<'_, T>, &Context)) -> Self {
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
    fn run(&self, world: &mut World, ctx: &mut Context) {
        let mut query = Query::new(world);
        query.build_matches();
        (self.func)(query, ctx);
    }
}

pub trait SystemFunction {
    fn run(&self, world: &mut World, ctx: &mut Context);
}
