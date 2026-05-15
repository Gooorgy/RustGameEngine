use crate::handle::Handle;
use crate::Guid;
use std::collections::HashMap;

pub struct TypedStore<T> {
    guid_to_handle: HashMap<Guid, Handle<T>>,
    handle_to_guid: HashMap<Handle<T>, Guid>,
    data: HashMap<Handle<T>, T>,
    next_id: u64,
}

impl<T> TypedStore<T> {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            guid_to_handle: HashMap::new(),
            handle_to_guid: HashMap::new(),
            data: HashMap::new(),
        }
    }
    
    pub fn get_or_insert(&mut self, guid: Guid, f: impl FnOnce() -> Option<T>) -> Option<Handle<T>> {
        if let Some(&handle) = self.guid_to_handle.get(&guid) {
            return Some(handle);
        }
        
        let value = f()?;
        let handle = Handle::new(self.next_id);
        self.next_id += 1;        
        self.guid_to_handle.insert(guid, handle);
        self.handle_to_guid.insert(handle, guid);
        self.data.insert(handle, value);
        Some(handle)
    }
    
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.data.get(&handle)
    }
}
