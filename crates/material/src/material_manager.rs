use crate::{Material, MaterialParameterBinding, MaterialParameterBindingData};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct MaterialHandle(usize);

pub struct MaterialManager {
    materials: Vec<Rc<RefCell<dyn Material>>>,
    shader_variants: HashMap<String, MaterialVariant>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: vec![],
            shader_variants: HashMap::new(),
        }
    }

    pub fn add_material_instance(&mut self, material: impl Material + 'static) -> MaterialHandle {
        let bindings = material.get_bindings();
        let push_constant_size = material.get_push_constants().len();
        let shader_path = material.get_shader_path();

        let shader_variant =
            Self::create_variant(bindings, push_constant_size, shader_path.clone());
        self.shader_variants.entry(shader_path).or_insert(shader_variant);

        let new_handle = self.materials.len();
        self.materials.push(Rc::new(RefCell::new(material)));

        MaterialHandle(new_handle)
    }

    pub fn get_variants(&self) -> Vec<&MaterialVariant> {
        self.shader_variants.values().collect::<Vec<_>>()
    }

    pub fn get_material_data(
        &self,
        material_handle: MaterialHandle,
    ) -> Vec<MaterialParameterBinding> {
        self.materials
            .get(material_handle.0)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", material_handle))
            .borrow()
            .get_bindings()
    }

    pub fn get_material_push_const_data(&self, material_handle: MaterialHandle) -> Vec<u8> {
        self.materials
            .get(material_handle.0)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", material_handle))
            .borrow()
            .get_push_constants()
            .to_owned()
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
                    binding_type: match binding.data {
                        MaterialParameterBindingData::Texture(_) => BindingType::ImageSampler,
                        _ => BindingType::ImageSampler,
                    },
                })
                .collect(),
        }
    }

    pub fn get_variant(&self, material_handle: MaterialHandle) -> &MaterialVariant {
        let material = self
            .materials
            .get(material_handle.0)
            .unwrap_or_else(|| panic!("MaterialManager: invalid handle {:?}", material_handle))
            .borrow()
            .get_shader_path();

        self.shader_variants
            .get(&material)
            .expect("Unknown material handle")
    }
}

impl Default for MaterialManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MaterialData {
    pub binding_data: Vec<MaterialParameterBinding>,
    pub variant_name: String,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct MaterialVariant {
    pub name: String,
    pub push_constant_size: usize,
    pub binding_info: Vec<ShaderBindingInfo>,
}

pub struct ShaderPushConstantInfo {
    pub size: usize,
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
