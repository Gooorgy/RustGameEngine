use crate::component::Component;
use crate::component::archetype::{Archetype, Column};
use crate::query::QueryParameter;
use std::any::TypeId;

impl<T1: Component> QueryParameter for &mut T1 {
    type Item<'w> = &'w mut T1;

    type MatchKey = usize;

    const COLUMN_COUNT: usize = 1;

    fn component_type() -> Vec<TypeId> {
        vec![TypeId::of::<T1>()]
    }

    fn check_match(archetype: &Archetype) -> Option<Self::MatchKey> {
        archetype.components.get(&TypeId::of::<T1>()).copied()
    }

    fn collect_columns(state: usize, columns_out: &mut Vec<usize>) {
        columns_out.push(state);
    }

    unsafe fn fetch<'w>(columns: &mut [*mut Column], row: usize) -> Self::Item<'w> {
        unsafe {
            let column = &mut *columns[0];
            let data = column.data.as_any_mut().downcast_mut::<Vec<T1>>().unwrap();

            &mut data[row]
        }
    }
}

macro_rules! impl_query_parameter {
    ($first:ident $(, $rest:ident)*) => {
        impl<$first: QueryParameter, $($rest: QueryParameter),*> QueryParameter for ($first, $($rest,)*) {
            type Item<'w> = ($first::Item<'w>, $($rest::Item<'w>,)*);

            // MatchKey is a tuple of each QueryParameter's MatchKey
            type MatchKey = ($first::MatchKey, $($rest::MatchKey,)*);

            const COLUMN_COUNT: usize = $first::COLUMN_COUNT $(+ $rest::COLUMN_COUNT)*;

            fn component_type() -> Vec<TypeId> {
                let mut types = Vec::new();
                types.extend($first::component_type());
                $(types.extend($rest::component_type());)*
                types
            }

            fn check_match(archetype: &Archetype) -> Option<Self::MatchKey> {
                Some((
                    $first::check_match(archetype)?,
                    $($rest::check_match(archetype)?,)*
                ))
            }

            #[allow(non_snake_case)]
            fn collect_columns(state: Self::MatchKey, columns_out: &mut Vec<usize>) {
                let ($first, $($rest,)*) = state;
                $first::collect_columns($first, columns_out);
                $($rest::collect_columns($rest, columns_out);)*
            }

            #[allow(non_snake_case)]
            unsafe fn fetch<'w>(columns: &mut [*mut Column], row: usize) -> Self::Item<'w> {
                let mut offset = 0;

                let $first = {
                    let slice = &mut columns[offset..offset + $first::COLUMN_COUNT];
                    offset += $first::COLUMN_COUNT;
                    $first::fetch(slice, row)
                };

                $(
                    let $rest = {
                        let slice = &mut columns[offset..offset + $rest::COLUMN_COUNT];
                        offset += $rest::COLUMN_COUNT;
                        $rest::fetch(slice, row)
                    };
                )*

                ($first, $($rest,)*)
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

impl_tuples!(impl_query_parameter);
