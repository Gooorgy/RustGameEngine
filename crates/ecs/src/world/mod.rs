use crate::component::archetype::{Archetype, ColumnFactory};
use crate::component::component_storage::ComponentInsertion;
use crate::entity::Entity;
use crate::query::{Query, QueryParameter};
use std::any::TypeId;
use std::collections::HashMap;

pub struct World {
    pub(crate) archetypes: Vec<Archetype>,
    pub(crate) archetype_index: HashMap<ArchetypeKey, ArchetypeId>,
    column_registry: ColumnRegistry,
    entity_meta: Vec<Option<EntityStorageData>>,
    free_list: Vec<usize>,
    //query_cache: HashMap<QueryKey, QueryCache>,
}

#[derive(Clone, Copy)]
pub(crate) struct ArchetypeId(pub(crate) usize);

struct ColumnRegistry {
    factories: HashMap<TypeId, ColumnFactory>,
}

impl ColumnRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn ensure(&mut self, type_id: TypeId, column_factory: ColumnFactory) {
        self.factories.entry(type_id).or_insert(column_factory);
    }

    pub fn get(&self, type_ids: &[TypeId]) -> Vec<(&ColumnFactory, TypeId)> {
        type_ids
            .iter()
            .map(|type_id| {
                let factory = self
                    .factories
                    .get(type_id)
                    .unwrap_or_else(|| panic!("Factory for type {:?} not registered", type_id));

                (factory, *type_id)
            })
            .collect::<Vec<_>>()
    }
}

pub struct EntityStorageData {
    pub archetype_id: ArchetypeId,
    pub row: usize,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ArchetypeKey {
    pub type_ids: Vec<TypeId>,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: vec![],
            archetype_index: HashMap::new(),
            column_registry: ColumnRegistry::new(),
            entity_meta: vec![],
            free_list: vec![],
            //query_cache: HashMap::new(),
        }
    }

    pub fn query<Q: QueryParameter>(&mut self) -> Query<'_, Q> {
        let mut query = Query::new(self);
        query.build_matches();
        query
    }

    #[allow(private_bounds)]
    pub fn create_entity(&mut self, components: impl ComponentInsertion) -> Entity {
        let id = self.free_list.pop().unwrap_or_else(|| {
            self.entity_meta.push(None);
            self.entity_meta.len() - 1
        });

        let mut values = vec![];
        let mut type_ids = vec![];
        components.for_each_component(|type_id, component_value, column_factory| {
            self.column_registry.ensure(type_id, column_factory);
            values.push(component_value);
            type_ids.push(type_id);
        });

        type_ids.sort_unstable();
        let key = ArchetypeKey { type_ids };

        let archetype_id = if let Some(&existing_index) = self.archetype_index.get(&key) {
            existing_index
        } else {
            let factories = self.column_registry.get(&key.type_ids);
            let archetype = Archetype::new(factories);
            let new_id = ArchetypeId(self.archetypes.len());
            self.archetypes.push(archetype);
            self.archetype_index.insert(key, new_id);
            new_id
        };
        let entity = Entity(id);
        let row = self.archetypes[archetype_id.0].insert(entity, values);
        self.entity_meta[id] = Some(EntityStorageData { archetype_id, row });

        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        let meta = self.entity_meta[entity.0].as_mut().unwrap();
        let archetype_id = meta.archetype_id;
        let row = meta.row;

        let archetype = &mut self.archetypes[archetype_id.0];
        if let Some(swapped) = archetype.remove(row) {
            self.entity_meta[swapped.0].as_mut().unwrap().row = row;
        }
        
        self.entity_meta[entity.0] = None;
        self.free_list.push(entity.0);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
