use crate::primitives::types::EnginePrimitiveType;
use core::types::transform::Transform;
use core::EngineContext;
use nalgebra_glm::Vec3;
use std::cell::RefCell;
use std::rc::Rc;

pub trait GameObject: GameObjectDefaults + HasGameObjectType {
    fn setup(&mut self, engine_context: &EngineContext);
}

pub trait GameObjectDefaults {
    fn get_transform(&self) -> Transform;

    fn with_transform(self, transform: Transform) -> Self
    where
        Self: Sized;
    fn with_location(self, location: Vec3) -> Self
    where
        Self: Sized;
    fn with_rotation(self, rotation: Vec3) -> Self
    where
        Self: Sized;
    fn with_scale(self, scale: Vec3) -> Self
    where
        Self: Sized;
}

pub trait HasGameObjectType {
    fn get_game_object_type(&self) -> GameObjectType;
    
}

pub trait HasGameObjectTypeImpl {
    const GAME_OBJECT_TYPE: GameObjectType;
}

impl<T> HasGameObjectType for T
where
    T: HasGameObjectTypeImpl,
{
    fn get_game_object_type(&self) -> GameObjectType {
        T::GAME_OBJECT_TYPE
    }
}

#[derive(Clone, Copy)]
pub enum GameObjectType {
    EnginePrimitive(EnginePrimitiveType),
    Custom,
}
