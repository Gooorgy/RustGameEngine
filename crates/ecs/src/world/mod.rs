use crate::command_buffer::{Command, Commands};
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
    pub(crate) entity_allocator: EntityAllocator,
}

/// Provides split access to archetypes and command recording without exposing World directly.
pub struct SystemAccess<'a> {
    pub archetypes: &'a mut Vec<Archetype>,
    pub commands: Commands<'a>,
}

impl<'a> SystemAccess<'a> {
    pub fn into_queue(self) -> Vec<Command> {
        self.commands.into_queue()
    }
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

pub(crate) struct EntityStorageData {
    pub(crate) archetype_id: ArchetypeId,
    pub(crate) row: usize,
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
            entity_allocator: EntityAllocator::new(),
            //query_cache: HashMap::new(),
        }
    }

    pub fn system_access(&mut self) -> SystemAccess<'_> {
        SystemAccess {
            archetypes: &mut self.archetypes,
            commands: Commands {
                queue: vec![],
                entity_allocator: &mut self.entity_allocator,
            },
        }
    }

    pub fn query<Q: QueryParameter>(&mut self) -> Query<'_, Q> {
        let mut query = Query::new(&mut self.archetypes);
        query.build_matches();
        query
    }

    #[allow(private_bounds)]
    pub fn create_entity(&mut self, components: impl ComponentInsertion) -> Entity {
        let entity = self.entity_allocator.reserve();

        self.create_reserved_entity(entity, components);

        entity
    }

    #[allow(private_bounds)]
    pub fn create_reserved_entity(&mut self, entity: Entity, components: impl ComponentInsertion) {
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
        let row = self.archetypes[archetype_id.0].insert(entity, values);
        self.entity_allocator.entity_meta[entity.0] = Some(EntityStorageData { archetype_id, row });
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        let meta = self.entity_allocator.entity_meta[entity.0]
            .as_mut()
            .unwrap();
        let archetype_id = meta.archetype_id;
        let row = meta.row;

        let archetype = &mut self.archetypes[archetype_id.0];
        if let Some(swapped) = archetype.remove(row) {
            self.entity_allocator.entity_meta[swapped.0]
                .as_mut()
                .unwrap()
                .row = row;
        }

        self.entity_allocator.entity_meta[entity.0] = None;
        self.entity_allocator.free_list.push(entity.0);
    }

    pub fn flush_queue(&mut self, queue: Vec<Command>) {
        for cmd in queue {
            match cmd {
                Command::SpawnEntity(_, inserter) => inserter(self),
                Command::DespawnEntity(entity) => self.remove_entity(entity),
            }
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct EntityAllocator {
    entity_meta: Vec<Option<EntityStorageData>>,
    free_list: Vec<usize>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            entity_meta: vec![],
            free_list: vec![],
        }
    }

    pub fn reserve(&mut self) -> Entity {
        let id = self.free_list.pop().unwrap_or_else(|| {
            self.entity_meta.push(None);
            self.entity_meta.len() - 1
        });
        Entity(id)
    }
}
