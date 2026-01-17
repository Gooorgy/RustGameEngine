use std::any::{Any, TypeId};

pub(crate) mod archetype;
pub mod component_storage;
mod impls;

use crate::component::archetype::{ColumnFactory, ComponentValue};
pub use ecs_macros::Component;

pub trait Component: Any {}

pub(crate) trait ComponentInsertion {
    fn for_each_component(self, f: impl FnMut(TypeId, ComponentValue, ColumnFactory));
}
