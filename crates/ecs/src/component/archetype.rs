use crate::component::Component;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::error::Error;

pub struct Archetype {
    pub components: HashMap<TypeId, usize>,
    current_row: usize,
    pub(crate) columns: Vec<Column>,
}

impl Archetype {
    pub fn new(factories: Vec<(&ColumnFactory, TypeId)>) -> Self {
        let mut columns = vec![];
        let mut components = HashMap::new();
        for (factory, type_id) in factories {
            let column_index = columns.len();
            columns.push(Column { data: factory() });
            components.insert(type_id, column_index);
        }

        Self {
            components,
            current_row: 0,
            columns,
        }
    }

    pub fn insert(&mut self, components: Vec<ComponentValue>) -> Option<usize> {
        for value in components {
            let type_id = value.type_id();
            let column = self.components[&type_id];
            self.columns[column].data.push_erased(value).ok()?;
        }
        let new_row = self.current_row;
        self.current_row += 1;

        Some(new_row)
    }

    // pub fn column<T>(&self, column_index: usize) -> Vec<T> {
    //     self.columns[column_index].data.pu
    // }
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
    pub data: Box<dyn ColumnData>,
}

pub(crate) trait ColumnData: Any {
    fn push_erased(&mut self, value: ComponentValue) -> Result<(), Box<dyn Error>>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize; // Add this method
}

impl<T: Component + 'static> ColumnData for Vec<T> {
    fn push_erased(&mut self, component_value: ComponentValue) -> Result<(), Box<dyn Error>> {
        let x = component_value.take::<T>().ok_or("Bad type!")?;
        self.push(x);

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
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
