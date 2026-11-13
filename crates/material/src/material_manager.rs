use crate::{Material, MaterialParameterBinding, ShaderRef};
use common::{Guid, Handle};
use std::collections::HashMap;

pub struct MaterialData;
pub type MaterialHandle = Handle<MaterialData>;

pub struct MaterialVariantData;
pub type MaterialVariantHandle = Handle<MaterialVariantData>;

pub struct MaterialManager {
    materials: Vec<MaterialRecord>,
    variants: Vec<MaterialVariant>,
    variant_index: HashMap<MaterialVariantKey, MaterialVariantHandle>,
    guid_index: HashMap<Guid, MaterialHandle>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: Vec::new(),
            variants: Vec::new(),
            variant_index: HashMap::new(),
            guid_index: HashMap::new(),
        }
    }

    /// Returns the cached handle if this GUID was already loaded, otherwise builds and registers it.
    pub fn get_or_insert(&mut self, guid: Guid, f: impl FnOnce() -> Material) -> MaterialHandle {
        if let Some(&handle) = self.guid_index.get(&guid) {
            return handle;
        }
        let handle = self.insert(f());
        self.guid_index.insert(guid, handle);
        handle
    }

    fn insert(&mut self, material: Material) -> MaterialHandle {
        let variant_handle = self.ensure_variant(&material);
        let handle = MaterialHandle::new(self.materials.len() as u64);
        self.materials.push(MaterialRecord {
            data: material,
            variant_handle,
        });
        handle
    }

    pub fn get_bindings(&self, handle: MaterialHandle) -> &[MaterialParameterBinding] {
        &self.material_record(handle).data.bindings
    }

    pub fn get_push_constants(&self, handle: MaterialHandle) -> &[u8] {
        &self.material_record(handle).data.push_constants
    }

    pub fn get_variant_handle(&self, handle: MaterialHandle) -> MaterialVariantHandle {
        self.material_record(handle).variant_handle
    }

    pub fn get_variant(&self, handle: MaterialVariantHandle) -> &MaterialVariant {
        self.variants
            .get(handle.raw() as usize)
            .unwrap_or_else(|| panic!("MaterialManager: invalid variant handle {:?}", handle))
    }

    fn material_record(&self, handle: MaterialHandle) -> &MaterialRecord {
        self.materials
            .get(handle.raw() as usize)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", handle))
    }

    fn ensure_variant(&mut self, material: &Material) -> MaterialVariantHandle {
        let key = MaterialVariantKey::from_material(material);
        if let Some(&handle) = self.variant_index.get(&key) {
            return handle;
        }

        let handle = MaterialVariantHandle::new(self.variants.len() as u64);
        self.variants.push(key.to_variant());
        self.variant_index.insert(key, handle);
        handle
    }
}

impl Default for MaterialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct MaterialVariantKey {
    vertex_shader: ShaderRef,
    fragment_shader: ShaderRef,
    active_defines: Vec<String>,
    push_constant_size: usize,
}

impl MaterialVariantKey {
    fn from_material(material: &Material) -> Self {
        Self {
            vertex_shader: material.vertex_shader.clone(),
            fragment_shader: material.fragment_shader.clone(),
            active_defines: material.active_defines.clone(),
            push_constant_size: material.push_constants.len(),
        }
    }

    fn to_variant(&self) -> MaterialVariant {
        MaterialVariant {
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            active_defines: self.active_defines.clone(),
            push_constant_size: self.push_constant_size,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MaterialVariant {
    pub vertex_shader: ShaderRef,
    pub fragment_shader: ShaderRef,
    pub active_defines: Vec<String>,
    pub push_constant_size: usize,
}

struct MaterialRecord {
    data: Material,
    variant_handle: MaterialVariantHandle,
}
