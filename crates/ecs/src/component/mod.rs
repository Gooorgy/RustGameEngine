use std::any::Any;

mod archetype;
pub mod component_storage;
pub mod query;

pub trait Component: Any {}

// impl<T1: 'static> Component for T1 {
//     fn insert(&mut self, entity: Entity, archetype: &mut Archetype) {
//         let type_id = TypeId::of::<T1>();
//     }
//
//     fn type_ids(&self) -> &[TypeId] {
//         todo!()
//     }
// }
