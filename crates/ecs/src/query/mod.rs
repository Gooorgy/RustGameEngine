mod impls;

use crate::component::archetype::{Archetype, Column};
use crate::world::ArchetypeId;
use std::any::TypeId;
use std::marker::PhantomData;

pub trait QueryParameter {
    type Item<'w>;

    type MatchKey: Copy;

    const COLUMN_COUNT: usize;

    fn component_type() -> Vec<TypeId>;

    fn check_match(archetype: &Archetype) -> Option<Self::MatchKey>;

    fn collect_columns(state: Self::MatchKey, columns_out: &mut Vec<usize>);

    /// Fetches component references for the given row.
    ///
    /// # Safety
    ///
    /// `columns` must be valid, non-aliased pointers for lifetime `'w`, and `row`
    /// must be within bounds for all referenced column slices.
    unsafe fn fetch<'w>(
        columns: &mut [*mut Column],
        row: usize,
    ) -> <Self as QueryParameter>::Item<'w>;
}

pub struct Match<Q: QueryParameter> {
    archetype_id: ArchetypeId,
    match_key: Q::MatchKey,
}

pub struct Query<'a, Q: QueryParameter> {
    pub(crate) archetypes: &'a mut Vec<Archetype>,
    pub(crate) matches: Vec<Match<Q>>,
}

impl<'a, Q: QueryParameter> Query<'a, Q> {
    pub fn new(archetypes: &'a mut Vec<Archetype>) -> Self {
        Self {
            archetypes,
            matches: Vec::new(),
        }
    }

    pub fn iter(&mut self) -> QueryIter<'_, Q> {
        QueryIter::new(&mut self.matches, self.archetypes)
    }

    pub fn build_matches(&mut self) {
        self.matches.clear();

        for (index, archetype) in self.archetypes.iter().enumerate() {
            if let Some(match_key) = Q::check_match(archetype) {
                self.matches.push(Match {
                    archetype_id: ArchetypeId(index),
                    match_key,
                });
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn refresh(&mut self, archetype_id: ArchetypeId) {
        self.matches.clear();

        let archetype = &self.archetypes[archetype_id.0];
        if let Some(match_key) = Q::check_match(archetype) {
            self.matches.push(Match {
                archetype_id,
                match_key,
            });
        }
    }
}

pub struct QueryIter<'w, Q: QueryParameter> {
    matches_iter: std::slice::IterMut<'w, Match<Q>>,
    current_archetype: Option<ArchetypeIter<'w, Q>>,
    world_archetypes: *mut Vec<Archetype>,
    _phantom: PhantomData<Q>,
}

impl<'w, Q: QueryParameter> QueryIter<'w, Q> {
    pub fn new(matches: &'w mut [Match<Q>], world_archetypes: *mut Vec<Archetype>) -> Self {
        Self {
            matches_iter: matches.iter_mut(),
            current_archetype: None,
            world_archetypes,
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
            if let Some(archetype_iter) = &mut self.current_archetype
                && archetype_iter.current_row < archetype_iter.total_rows
            {
                let row = archetype_iter.current_row;
                archetype_iter.current_row += 1;

                // SAFETY: column_ptrs are valid for 'w lifetime
                // and row is within bounds
                return Some(unsafe { Q::fetch(&mut archetype_iter.column_ptrs, row) });
            }

            // SAFETY: world_data pointer is valid for 'w lifetime
            // archetype_key is guaranteed to exist
            unsafe {
                let column_match = self.matches_iter.next()?;

                let archetype = self.world_archetypes
                    .as_mut()
                    .expect("null archetypes pointer")
                    .get_mut(column_match.archetype_id.0)
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
