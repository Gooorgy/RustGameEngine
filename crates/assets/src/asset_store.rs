use crate::emesh::read_emesh;
use crate::etex::read_etex;
use crate::read_spv;
use common::{Guid, Handle, ImageData, ImageHandle, MeshData, MeshHandle, ShaderData, ShaderHandle, TypedStore};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::Path;

pub struct AssetStore {
    stores: HashMap<TypeId, Box<dyn Any>>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
        }
    }

    fn store_for_mut<T: 'static>(&mut self) -> &mut TypedStore<T> {
        self.stores
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(TypedStore::<T>::new()))
            .downcast_mut::<TypedStore<T>>()
            .unwrap()
    }

    fn store_for<T: 'static>(&self) -> Option<&TypedStore<T>> {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<TypedStore<T>>())
    }

    /// Loads a cooked `.emesh` file. Returns a cached handle if the same path
    /// was already loaded.
    pub fn load_mesh(&mut self, path: &Path, guid: Guid) -> Option<MeshHandle> {
        self.store_for_mut::<MeshData>()
            .get_or_insert(guid, || read_emesh(path).ok())
    }

    /// Loads a cooked `.etex` file. Returns a cached handle if the same path
    /// was already loaded.
    pub fn load_texture(&mut self, path: &Path, guid: Guid) -> Option<ImageHandle> {
        self.store_for_mut::<ImageData>()
            .get_or_insert(guid, || read_etex(path).ok())
    }

    /// Loads a compiled `.spv` shader file. Returns a cached handle if the same path
    /// was already loaded.
    pub fn load_shader(&mut self, path: &Path, guid: Guid) -> Option<ShaderHandle> {
        self.store_for_mut::<ShaderData>()
            .get_or_insert(guid, || read_spv(path).ok())
    }

    pub fn get<T: 'static>(&self, handle: Handle<T>) -> Option<&T> {
        self.store_for::<T>()?.get(handle)
    }
}
