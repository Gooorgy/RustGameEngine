use assets::ImageHandle;
use nalgebra_glm::{vec4, Vec4};
use serde::Serialize;

pub trait Material {
    fn get_bindings(&self) -> Vec<MaterialParameterBinding>;
    fn get_push_constants(&self) -> &[u8];
    fn get_permutation_feature(&self) -> u32;
    fn get_shader_path(&self) -> String;
}

pub struct PbrMaterial {
    pub base_color: MaterialColorParameter,
    pub normal: MaterialColorParameter,
    pub ambient_occlusion: MaterialParameter,
    pub metallic: MaterialParameter,
    pub roughness: MaterialParameter,
    pub specular: MaterialParameter,
}

impl Material for PbrMaterial {
    fn get_bindings(&self) -> Vec<MaterialParameterBinding> {
        let base_color_binding = match self.base_color {
            MaterialColorParameter::Handle(handle) => {
                let material_parameter_binding = MaterialParameterBinding {
                    index: 0,
                    data: MaterialParameterBindingData::Texture(handle),
                };
                Some(material_parameter_binding)
            }
            _ => None,
        };

        let normal_binding = match self.normal {
            MaterialColorParameter::Handle(handle) => {
                let material_parameter_binding = MaterialParameterBinding {
                    index: 1,
                    data: MaterialParameterBindingData::Texture(handle),
                };
                Some(material_parameter_binding)
            }
            _ => None,
        };

        let packed_texture = if self.metallic.as_handle().is_some()
            || self.roughness.as_handle().is_some()
            || self.ambient_occlusion.as_handle().is_some()
            || self.specular.as_handle().is_some()
        {
            let packed = PackedTextureData {
                channel_r: self.ambient_occlusion.as_handle(),
                channel_g: self.metallic.as_handle(),
                channel_b: self.roughness.as_handle(),
                channel_a: self.specular.as_handle(),
            };

            let material_parameter_binding = MaterialParameterBinding {
                index: 2,
                data: MaterialParameterBindingData::PackedTexture(packed),
            };

            Some(material_parameter_binding)
        } else {
            None
        };

        [base_color_binding, normal_binding, packed_texture]
            .into_iter()
            .filter_map(|parameter| parameter)
            .collect::<Vec<_>>()
    }

    fn get_push_constants(&self) -> &[u8] {
        let data = PbrPushConstants {
            base_color: self.base_color.as_constant(vec4(0.0, 0.0, 0.0, 0.0)),
            normal: self.normal.as_constant(vec4(0.0, 0.0, 0.0, 0.0)),
            roughness: self.roughness.as_constant(1.0),
            specular: self.roughness.as_constant(0.0),
            metallic: self.metallic.as_constant(0.0),
            ambient_occlusion: self.ambient_occlusion.as_constant(0.0),
        };

        let bytes = unsafe {
            ::core::slice::from_raw_parts(
                (&data as *const PbrPushConstants) as *const u8,
                ::core::mem::size_of::<PbrPushConstants>(),
            )
        };

        bytes
    }

    fn get_permutation_feature(&self) -> u32 {
        let mut pbr_features = PbrFeatures::NONE;

        if self.base_color.as_handle().is_some() {
            pbr_features |= PbrFeatures::HAS_COLOR_TEXTURE
        };

        if self.normal.as_handle().is_some() {
            pbr_features |= PbrFeatures::HAS_NORMAL_TEXTURE
        };

        if self.ambient_occlusion.as_handle().is_some() {
            pbr_features |= PbrFeatures::HAS_ORM_TEXTURE
        };

        if self.metallic.as_handle().is_some() {
            pbr_features |= PbrFeatures::HAS_ORM_TEXTURE
        };

        if self.roughness.as_handle().is_some() {
            pbr_features |= PbrFeatures::HAS_ORM_TEXTURE
        };

        if self.specular.as_handle().is_some() {
            pbr_features |= PbrFeatures::HAS_ORM_TEXTURE
        };

        pbr_features.bits()
    }

    fn get_shader_path(&self) -> String {
        let base_name = "pbr.frag";
        let mut feature_name = vec![];
        let features = PbrFeatures::from_bits(self.get_permutation_feature())
            .expect("Failed to get permutation feature");

        if features.contains(PbrFeatures::HAS_ORM_TEXTURE) {
            feature_name.push("orm");
        }

        if (features.contains(PbrFeatures::HAS_NORMAL_TEXTURE)) {
            feature_name.push("normal");
        }

        if features.contains(PbrFeatures::HAS_COLOR_TEXTURE) {
            feature_name.push("color");
        }

        if (feature_name.is_empty()) {
            return format!("{}.{}", base_name, "base",);
        }

        let combined_feature_name = feature_name.join(".");

        format!("{}.{}", base_name, combined_feature_name)
    }
}

#[repr(C)]
#[derive(Serialize)]
pub struct PbrPushConstants {
    base_color: Vec4,
    normal: Vec4,
    ambient_occlusion: f32,
    metallic: f32,
    roughness: f32,
    specular: f32,
}

pub enum MaterialParameter {
    Handle(ImageHandle),
    Constant(f32),
}

impl MaterialParameter {
    fn as_handle(&self) -> Option<ImageHandle> {
        match self {
            MaterialParameter::Handle(handle) => Some(*handle),
            _ => None,
        }
    }

    fn as_constant(&self, default: f32) -> f32 {
        match self {
            MaterialParameter::Constant(constant) => *constant,
            _ => default,
        }
    }
}

pub enum MaterialColorParameter {
    Handle(ImageHandle),
    Constant(Vec4),
}

impl MaterialColorParameter {
    fn as_handle(&self) -> Option<ImageHandle> {
        match self {
            MaterialColorParameter::Handle(handle) => Some(*handle),
            _ => None,
        }
    }

    fn as_constant(&self, default: Vec4) -> Vec4 {
        match self {
            MaterialColorParameter::Constant(constant) => *constant,
            _ => default,
        }
    }
}

pub struct MaterialParameterBinding {
    pub index: usize,
    pub data: MaterialParameterBindingData,
}

pub enum MaterialParameterBindingData {
    Texture(ImageHandle),
    PackedTexture(PackedTextureData),
}

pub struct PackedTextureData {
    channel_r: Option<ImageHandle>,
    channel_g: Option<ImageHandle>,
    channel_b: Option<ImageHandle>,
    channel_a: Option<ImageHandle>,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PbrFeatures: u32 {
        const NONE = 0;
        const HAS_COLOR_TEXTURE = 1 << 0;
        const HAS_NORMAL_TEXTURE = 1 << 1;
        const HAS_ORM_TEXTURE = 1 << 2;
    }
}
