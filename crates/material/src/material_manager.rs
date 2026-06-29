use crate::{Material, MaterialParameterBinding, ShaderRef};
use common::{Guid, Handle};
use std::collections::HashMap;

/// Marker type for material handles.
pub struct MaterialData;
pub type MaterialHandle = Handle<MaterialData>;

pub struct MaterialManager {
    materials: HashMap<MaterialHandle, Material>,
    shader_variants: HashMap<String, MaterialVariant>,
    guid_index: HashMap<Guid, MaterialHandle>,
    next_id: u64,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            shader_variants: HashMap::new(),
            guid_index: HashMap::new(),
            next_id: 0,
        }
    }

    /// Returns the cached handle if this GUID was already loaded, otherwise
    /// calls `f` to build the material, registers it, and caches the handle.
    pub fn get_or_insert(&mut self, guid: Guid, f: impl FnOnce() -> Material) -> MaterialHandle {
        if let Some(&handle) = self.guid_index.get(&guid) {
            return handle;
        }
        let handle = self.insert(f());
        self.guid_index.insert(guid, handle);
        handle
    }

    fn insert(&mut self, material: Material) -> MaterialHandle {
        let key = variant_key(
            &material.vertex_shader,
            &material.fragment_shader,
            &material.active_defines,
        );
        let variant = Self::create_variant(&material);
        self.shader_variants.entry(key).or_insert(variant);

        let handle = Handle::new(self.next_id);
        self.next_id += 1;
        self.materials.insert(handle, material);
        handle
    }

    pub fn get_variants(&self) -> Vec<&MaterialVariant> {
        self.shader_variants.values().collect()
    }

    pub fn get_bindings(&self, handle: MaterialHandle) -> &[MaterialParameterBinding] {
        &self
            .materials
            .get(&handle)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", handle))
            .bindings
    }

    pub fn get_push_constants(&self, handle: MaterialHandle) -> &[u8] {
        &self
            .materials
            .get(&handle)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", handle))
            .push_constants
    }

    pub fn get_variant(&self, handle: MaterialHandle) -> &MaterialVariant {
        let material = self
            .materials
            .get(&handle)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", handle));

        let key = variant_key(
            &material.vertex_shader,
            &material.fragment_shader,
            &material.active_defines,
        );
        self.shader_variants
            .get(&key)
            .expect("MaterialManager: shader variant not found")
    }

    fn create_variant(material: &Material) -> MaterialVariant {
        MaterialVariant {
            vertex_shader: material.vertex_shader.clone(),
            fragment_shader: material.fragment_shader.clone(),
            active_defines: material.active_defines.clone(),
            push_constant_size: material.push_constants.len(),
            binding_info: material
                .bindings
                .iter()
                .map(|b| ShaderBindingInfo {
                    index: b.index,
                    binding_type: BindingType::ImageSampler,
                })
                .collect(),
        }
    }
}

impl Default for MaterialManager {
    fn default() -> Self {
        Self::new()
    }
}

fn variant_key(v: &ShaderRef, f: &ShaderRef, defines: &[String]) -> String {
    let s = |r: &ShaderRef| match r {
        ShaderRef::BuiltIn(n) => format!("b:{}", n),
        ShaderRef::Asset(g) => format!("a:{}", g),
    };
    format!("{}|{}|{}", s(v), s(f), defines.join(","))
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MaterialVariant {
    pub vertex_shader: ShaderRef,
    pub fragment_shader: ShaderRef,
    pub active_defines: Vec<String>,
    pub push_constant_size: usize,
    pub binding_info: Vec<ShaderBindingInfo>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ShaderBindingInfo {
    pub index: usize,
    pub binding_type: BindingType,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum BindingType {
    ImageSampler,
}
