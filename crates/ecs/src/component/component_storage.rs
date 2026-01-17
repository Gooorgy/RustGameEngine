use crate::component::archetype::{Archetype, ColumnFactory, ComponentValue, HasColumnFactory};
use crate::component::query::{Query, QueryParameter};
use crate::component::Component;
use crate::systems::SystemFunction;
use crate::Entity::Entity;
use std::any::TypeId;
use std::collections::HashMap;
use std::hash::Hash;

pub struct World {
    pub(crate) data: HashMap<ArchetypeKey, Archetype>,
    column_registry: ColumnRegistry,
    entities: Vec<Entity>,
    sparse_data: Vec<Option<EntityStorageData>>,
    systems: Vec<Box<dyn SystemFunction>>,
    //query_cache: HashMap<QueryKey, QueryCache>,
    archetype_gen: u64,
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
                    .expect(format!("Factory for type {:?} not registered", type_id).as_str());

                (factory, type_id.clone())
            })
            .collect::<Vec<_>>()
    }
}

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
            sparse_data: vec![],
            systems: vec![],
            archetype_gen: 0,
            //query_cache: HashMap::new(),
        }
    }

    pub fn Query<Q: QueryParameter>(&mut self) -> Query<Q> {
        todo!();
        // Query {
        //     world: self,
        //
        // }

        // let key = QueryKey::of::<Q>();
        //
        // let cache = self
        //     .query_cache
        //     .entry(key)
        //     .or_insert_with(|| QueryCache::default());
        //
        // if cache.generation != self.archetype_gen {}
    }

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

    pub fn update(&mut self) {
        let systems = std::mem::take(&mut self.systems);

        for system in &systems {
            system.run(self);
        }

        self.systems = systems;
    }

    pub fn register_system(&mut self, system: Box<dyn SystemFunction>) {
        self.systems.push(system);
    }
}

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
