mod impls;

use crate::component::archetype::{Archetype, Column};
use crate::world::{ArchetypeKey, World};
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

pub trait QueryParameter {
    type Item<'w>;

    type MatchKey: Copy;

    const COLUMN_COUNT: usize;

    fn component_type() -> Vec<TypeId>;

    fn check_match(archetype: &Archetype) -> Option<Self::MatchKey>;

    fn collect_columns(state: Self::MatchKey, columns_out: &mut Vec<usize>);

    unsafe fn fetch<'w>(
        columns: &mut [*mut Column],
        row: usize,
    ) -> <Self as QueryParameter>::Item<'w>;
}

pub struct Match<Q: QueryParameter> {
    archetype_key: ArchetypeKey,
    match_key: Q::MatchKey,
}

pub struct Query<'a, Q: QueryParameter> {
    pub(crate) world: &'a mut World,
    pub(crate) matches: Vec<Match<Q>>,
}

impl<Q: QueryParameter> Query<'_, Q> {
    pub fn iter(&mut self) -> QueryIter<Q> {
        QueryIter::new(&mut self.matches, &mut self.world.data as *mut _)
    }

    pub fn build_matches(&mut self) {
        self.matches.clear();

        for (archetype_key, archetype) in &self.world.data {
            if let Some(match_key) = Q::check_match(&archetype) {
                self.matches.push(Match {
                    match_key,
                    archetype_key: archetype_key.clone(),
                })
            }
        }
    }
    pub fn refresh(&mut self, archetype: &Archetype) {
        self.matches.clear();

        if let Some(match_key) = Q::check_match(archetype) {
            self.matches.push(Match {
                match_key,
                archetype_key: ArchetypeKey {
                    type_ids: Q::component_type(),
                },
            });
        }
    }
}

pub struct QueryIter<'w, Q: QueryParameter> {
    matches_iter: std::slice::IterMut<'w, Match<Q>>,
    current_archetype: Option<ArchetypeIter<'w, Q>>,
    world_data: *mut HashMap<ArchetypeKey, Archetype>,
    _phantom: PhantomData<Q>,
}

impl<'w, Q: QueryParameter> QueryIter<'w, Q> {
    pub fn new(
        matches: &'w mut [Match<Q>],
        world_data: *mut HashMap<ArchetypeKey, Archetype>,
    ) -> Self {
        Self {
            matches_iter: matches.iter_mut(),
            current_archetype: None,
            world_data,
            _phantom: PhantomData,
        }
    }
}

struct ArchetypeIter<'w, Q: QueryParameter> {
    column_ptrs: Vec<*mut Column>,
    current_row: usize,
    total_rows: usize,
    _phantom: PhantomData<&'w mut Q>,
}

impl<'w, Q: QueryParameter> Iterator for QueryIter<'w, Q> {
    type Item = Q::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to get next item from current archetype
            if let Some(archetype_iter) = &mut self.current_archetype {
                if archetype_iter.current_row < archetype_iter.total_rows {
                    let row = archetype_iter.current_row;
                    archetype_iter.current_row += 1;

                    // SAFETY: column_ptrs are valid for 'w lifetime
                    // and row is within bounds
                    return Some(unsafe { Q::fetch(&mut archetype_iter.column_ptrs, row) });
                }
            }

            // SAFETY: world_data pointer is valid for 'w lifetime
            // archetype_key is guaranteed to exist
            unsafe {
                let column_match = self.matches_iter.next()?;

                let archetype = (*self.world_data)
                    .get_mut(&column_match.archetype_key)
                    .expect("Archetype not registered");

                let total_rows = if archetype.columns.is_empty() {
                    0
                } else {
                    archetype.columns[0].data.len()
                };

                let mut column_indices = Vec::with_capacity(Q::COLUMN_COUNT);
                Q::collect_columns(column_match.match_key, &mut column_indices);

                let column_ptrs = column_indices
                    .into_iter()
                    .map(|index| &mut archetype.columns[index] as *mut Column)
                    .collect::<Vec<_>>();

                self.current_archetype = Some(ArchetypeIter {
                    column_ptrs,
                    current_row: 0,
                    total_rows,
                    _phantom: PhantomData,
                });
            }
        }
    }
}

