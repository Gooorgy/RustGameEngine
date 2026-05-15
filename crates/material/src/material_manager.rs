use crate::{Material, MaterialParameterBinding};
use common::Handle;
use std::collections::HashMap;

/// Marker type for material handles.
pub struct MaterialData;
pub type MaterialHandle = Handle<MaterialData>;

pub struct MaterialManager {
    materials: HashMap<MaterialHandle, Box<dyn Material>>,
    shader_variants: HashMap<String, MaterialVariant>,
    next_id: u64,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            shader_variants: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_material_instance(&mut self, material: impl Material + 'static) -> MaterialHandle {
        let bindings = material.get_bindings();
        let push_constant_size = material.get_push_constants().len();
        let shader_path = material.get_shader_path();

        let variant = Self::create_variant(bindings, push_constant_size, shader_path.clone());
        self.shader_variants.entry(shader_path).or_insert(variant);

        let handle = Handle::new(self.next_id);
        self.next_id += 1;
        self.materials.insert(handle, Box::new(material));
        handle
    }

    pub fn get_variants(&self) -> Vec<&MaterialVariant> {
        self.shader_variants.values().collect()
    }

    pub fn get_material_data(&self, handle: MaterialHandle) -> Vec<MaterialParameterBinding> {
        self.materials
            .get(&handle)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", handle))
            .get_bindings()
    }

    pub fn get_material_push_const_data(&self, handle: MaterialHandle) -> Vec<u8> {
        self.materials
            .get(&handle)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", handle))
            .get_push_constants()
            .to_owned()
    }

    pub fn get_variant(&self, handle: MaterialHandle) -> &MaterialVariant {
        let shader_path = self.materials
            .get(&handle)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", handle))
            .get_shader_path();

        self.shader_variants
            .get(&shader_path)
            .expect("MaterialManager: shader variant not found")
    }

    fn create_variant(
        bindings: Vec<MaterialParameterBinding>,
        push_constant_size: usize,
        path: String,
    ) -> MaterialVariant {
        MaterialVariant {
            name: path,
            push_constant_size,
            binding_info: bindings
                .iter()
                .map(|binding| ShaderBindingInfo {
                    index: binding.index,
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

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct MaterialVariant {
    pub name: String,
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