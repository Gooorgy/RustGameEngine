use assets::ImageHandle;
use bitflags::Bits;
use nalgebra_glm::{vec4, DVec1, Scalar, Vec4};
use serde::Serialize;

pub trait Material {
    fn get_bindings(&self) -> Vec<MaterialParameterBinding>;
    fn get_push_constants(&self) -> PbrPushConstants;

    fn get_permutation_feature(&self) -> u32;
}

pub struct PbrMaterial {
    base_color: MaterialColorParameter,
    normal: MaterialColorParameter,
    ambient_occlusion: MaterialParameter,
    metallic: MaterialParameter,
    roughness: MaterialParameter,
    specular: MaterialParameter,
    test: Vec4,
}

impl Material for PbrMaterial {
    fn get_bindings(&self) -> Vec<MaterialParameterBinding> {
        let base_color_binding = match self.base_color {
            MaterialColorParameter::Handle(handle) => {
                Some(MaterialParameterBinding::texture(0, handle))
            }
            _ => None,
        };

        let normal_binding = match self.normal {
            MaterialColorParameter::Handle(handle) => {
                Some(MaterialParameterBinding::texture(1, handle))
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

            Some(MaterialParameterBinding::packed_texture(3, packed))
        } else {
            None
        };

        [base_color_binding, normal_binding, packed_texture]
            .into_iter()
            .filter_map(|parameter| parameter)
            .collect::<Vec<_>>()
    }

    fn get_push_constants(&self) -> PbrPushConstants {
        PbrPushConstants {
            base_color: self.base_color.as_constant(vec4(0.0, 0.0, 0.0, 0.0)),
            normal: self.normal.as_constant(vec4(0.0, 0.0, 0.0, 0.0)),
            roughness: self.roughness.as_constant(1.0),
            specular: self.roughness.as_constant(0.0),
            metallic: self.metallic.as_constant(0.0),
            ambient_occlusion: self.ambient_occlusion.as_constant(0.0),
        }
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
}

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

pub enum MaterialParameterBinding {
    Texture(MaterialParameterBindingData<ImageHandle>),
    PackedTexture(MaterialParameterBindingData<PackedTextureData>),
}

impl MaterialParameterBinding {
    fn texture(index: u32, image_handle: ImageHandle) -> Self {
        MaterialParameterBinding::Texture(MaterialParameterBindingData {
            binding_index: index,
            binding_data: image_handle,
        })
    }

    fn packed_texture(index: u32, data: PackedTextureData) -> Self {
        MaterialParameterBinding::PackedTexture(MaterialParameterBindingData {
            binding_index: index,
            binding_data: data,
        })
    }
}

pub struct MaterialParameterBindingData<T> {
    binding_index: u32,
    binding_data: T,
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
