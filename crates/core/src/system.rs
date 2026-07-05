use crate::asset_context::AssetContext;
use common::{Guid, MeshHandle};
use ecs::command_buffer::Commands;
use ecs::component::archetype::Archetype;
use ecs::query::{Query, QueryParameter};
use input::InputManager;
use material::material_manager::{MaterialHandle, MaterialManager};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct Context<'a> {
    pub dt: f32,
    pub assets: &'a mut AssetContext,
    pub material_manager: &'a mut MaterialManager,
    pub input: &'a InputManager,
    pub custom: &'a HashMap<TypeId, Rc<RefCell<dyn Any>>>,
}

impl<'a> Context<'a> {
    /// Loads a mesh by GUID, returning the cached handle if already loaded.
    pub fn load_mesh(&mut self, guid: Guid) -> MeshHandle {
        self.assets.load_mesh(guid)
    }

    /// Loads a material by GUID, returning the cached handle if already loaded.
    pub fn load_material(&mut self, guid: Guid) -> MaterialHandle {
        let assets = &mut self.assets;
        self.material_manager
            .get_or_insert(guid, || assets.build_material(guid))
    }
}

pub struct System<T: 'static + QueryParameter> {
    func: fn(Query<'_, T>, &mut Context, &mut Commands),
    _phantom: PhantomData<T>,
}

impl<T: 'static + QueryParameter> System<T> {
    pub fn new(func: fn(Query<'_, T>, &mut Context, &mut Commands)) -> Self {
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
    fn run(&self, archetypes: &mut Vec<Archetype>, ctx: &mut Context, commands: &mut Commands) {
        let mut query = Query::new(archetypes);
        query.build_matches();
        (self.func)(query, ctx, commands);
    }
}

pub trait SystemFunction {
    fn run(&self, archetypes: &mut Vec<Archetype>, ctx: &mut Context, commands: &mut Commands);
}