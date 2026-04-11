use crate::component::archetype::{Archetype, ColumnFactory};
use crate::component::component_storage::ComponentInsertion;
use crate::entity::Entity;
use crate::query::{Query, QueryParameter};
use crate::systems::{ManagerContext, SystemFunction};
use std::any::TypeId;
use std::collections::HashMap;

pub struct World {
    pub(crate) data: HashMap<ArchetypeKey, Archetype>,
    column_registry: ColumnRegistry,
    entities: Vec<Entity>,
    systems: Vec<Box<dyn SystemFunction>>,
    //query_cache: HashMap<QueryKey, QueryCache>,
}

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

#[allow(dead_code)]
pub struct EntityStorageData {
    pub archetype_key: ArchetypeKey,
    pub row: usize,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ArchetypeKey {
    pub type_ids: Vec<TypeId>,
}

impl World {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            column_registry: ColumnRegistry::new(),
            entities: vec![],
            systems: vec![],
            //query_cache: HashMap::new(),
        }
    }

    pub fn query<Q: QueryParameter>(&mut self) -> Query<'_, Q> {
        let mut query = Query {
            world: self,
            matches: vec![],
        };
        query.build_matches();
        query
    }

    #[allow(private_bounds)]
    pub fn create_entity(&mut self, components: impl ComponentInsertion) -> Entity {
        // this works until Entities are removable. leave this for now...
        let index = self.entities.len();
        self.entities.push(Entity(index));

        let mut values = vec![];
        let mut type_ids = vec![];
        components.for_each_component(|type_id, component_value, column_factory| {
            self.column_registry.ensure(type_id, column_factory);

            values.push(component_value);
            type_ids.push(type_id);
        });

        let key = ArchetypeKey {
            type_ids: type_ids.clone(),
        };

        let archetype = self
            .data
            .entry(key)
            .or_insert_with(|| Archetype::new(self.column_registry.get(type_ids.as_slice())));

        archetype.insert(values);

        Entity(index)
    }

    pub fn update(&mut self, ctx: &ManagerContext) {
        let systems = std::mem::take(&mut self.systems);

        for system in &systems {
            system.run(self, ctx);
        }

        self.systems = systems;
    }

    pub fn register_system(&mut self, system: Box<dyn SystemFunction>) {
        self.systems.push(system);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
