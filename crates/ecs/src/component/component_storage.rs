use crate::component::archetype::{ColumnFactory, ComponentValue};
use std::any::TypeId;

/// Implemented for any tuple of components so they can be passed to `World::create_entity`.
/// All tuple sizes from 1 to 12 are covered by the macro impls in `impls.rs`.
pub(crate) trait ComponentInsertion {
    fn for_each_component(self, f: impl FnMut(TypeId, ComponentValue, ColumnFactory));
}

