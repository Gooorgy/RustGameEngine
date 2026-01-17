use crate::component::archetype::{ColumnFactory, ComponentValue, HasColumnFactory};
use crate::component::Component;
use std::any::TypeId;

pub(crate) trait ComponentInsertion {
    fn for_each_component(self, f: impl FnMut(TypeId, ComponentValue, ColumnFactory));
}

impl<T: Component + HasColumnFactory> ComponentInsertion for T {
    fn for_each_component(self, mut f: impl FnMut(TypeId, ComponentValue, ColumnFactory)) {
        f(
            TypeId::of::<T>(),
            ComponentValue::new(self),
            Self::get_factory(),
        )
    }
}

impl<T1: Component, T2: Component> ComponentInsertion for (T1, T2) {
    fn for_each_component(self, mut f: impl FnMut(TypeId, ComponentValue, ColumnFactory)) {
        f(
            TypeId::of::<T1>(),
            ComponentValue::new(self.0),
            T1::get_factory(),
        );
        f(
            TypeId::of::<T2>(),
            ComponentValue::new(self.1),
            T2::get_factory(),
        );
    }
}
