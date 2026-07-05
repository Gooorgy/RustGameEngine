use std::any::Any;

pub mod archetype;
pub mod component_storage;
mod impls;

pub use ecs_macros::Component;

pub trait Component: Any {}
