use std::any::TypeId;
use crate::component::archetype::{ColumnFactory, ComponentValue, HasColumnFactory};
use crate::component::{Component, ComponentInsertion};

macro_rules! impl_component_insertion {
    ($($t:ident),*) => {
        impl<$($t: Component + HasColumnFactory),*> ComponentInsertion for ($($t,)*) {
            #[allow(non_snake_case)]
            fn for_each_component(self, mut f: impl FnMut(TypeId, ComponentValue, ColumnFactory)) {
                let ($($t,)*) = self;
                $(
                    f(
                        TypeId::of::<$t>(),
                        ComponentValue::new($t),
                        $t::get_factory(),
                    );
                )*
            }
        }
    };
}

macro_rules! impl_tuples {
    ($macro:ident) => {
        $macro!(T1);
        $macro!(T1, T2);
        $macro!(T1, T2, T3);
        $macro!(T1, T2, T3, T4);
        $macro!(T1, T2, T3, T4, T5);
        $macro!(T1, T2, T3, T4, T5, T6);
        $macro!(T1, T2, T3, T4, T5, T6, T7);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
    };
}

impl_tuples!(impl_component_insertion);
