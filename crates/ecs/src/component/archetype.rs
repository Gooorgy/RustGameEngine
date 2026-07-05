use crate::component::Component;
use crate::entity::Entity;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::error::Error;

pub struct Archetype {
    pub components: HashMap<TypeId, usize>,
    pub(crate) columns: Vec<Column>,
    entities: Vec<Entity>,
}

impl Archetype {
    pub(crate) fn new(factories: Vec<(&ColumnFactory, TypeId)>) -> Self {
        let mut columns = vec![];
        let mut components = HashMap::new();
        for (factory, type_id) in factories {
            let column_index = columns.len();
            columns.push(Column { data: factory() });
            components.insert(type_id, column_index);
        }

        Self {
            components,
            columns,
            entities: vec![],
        }
    }

    pub fn insert(&mut self, entity: Entity, components: Vec<ComponentValue>) -> usize {
        for value in components {
            let type_id = value.type_id();
            let column = self.components[&type_id];
            self.columns[column]
                .data
                .push_erased(value)
                .expect("type mismatch on insert! archetype key is wrong");
        }
        self.entities.push(entity);
        self.entities.len() - 1
    }

    pub fn remove(&mut self, row: usize) -> Option<Entity> {
        for column in &mut self.columns {
            column.data.swap_remove_erased(row);
        }
        self.entities.swap_remove(row);
        (row < self.entities.len()).then(|| self.entities[row])
    }
}

pub struct ComponentValue {
    type_id: TypeId,
    value: Box<dyn Any>,
}

impl ComponentValue {
    pub fn new<T: Component>(component: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            value: Box::new(component),
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn take<T: Component + 'static>(self) -> Option<T> {
        self.value.downcast::<T>().ok().map(|boxed| *boxed)
    }
}

pub struct Column {
    pub(crate) data: Box<dyn ColumnData>,
}

pub(crate) trait ColumnData: Any {
    fn push_erased(&mut self, value: ComponentValue) -> Result<(), Box<dyn Error>>;
    fn swap_remove_erased(&mut self, row: usize);
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
}

impl<T: Component + 'static> ColumnData for Vec<T> {
    fn push_erased(&mut self, component_value: ComponentValue) -> Result<(), Box<dyn Error>> {
        let x = component_value.take::<T>().ok_or("Bad type!")?;
        self.push(x);

        Ok(())
    }

    fn swap_remove_erased(&mut self, row: usize) {
        self.swap_remove(row);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn len(&self) -> usize {
        self.len()
    }
}

pub(crate) type ColumnFactory = fn() -> Box<dyn ColumnData>;

pub(crate) trait HasColumnFactory {
    fn get_factory() -> ColumnFactory;
}

impl<T: Component> HasColumnFactory for T {
    fn get_factory() -> ColumnFactory {
        || Box::new(Vec::<T>::new())
    }
}
