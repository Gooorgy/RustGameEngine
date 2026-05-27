use common::{Guid, ImageHandle};
use nalgebra_glm::{vec4, Vec4};
use serde::Serialize;

/// Identifies a shader for one stage of a material pass.
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ShaderRef {
    /// Engine built-in, loaded by name from the engine shader directory. No GUID, no asset registry.
    BuiltIn(String),
    /// User shader asset cooked by the conditioner. GUID identifies the manifest; active defines select the variant.
    Asset(Guid),
}

/// The runtime representation of a material instance.
///
/// All material types (PBR, custom) reduce to this struct after build time.
/// The renderer uses `vertex_shader`, `fragment_shader`, and `active_defines` to
/// select and cache the right pipeline, and `bindings` + `push_constants` for per-draw data.
pub struct Material {
    pub vertex_shader: ShaderRef,
    pub fragment_shader: ShaderRef,
    /// Permutation defines active for this instance. Matched against the shader manifest to select the compiled variant.
    pub active_defines: Vec<String>,
    pub bindings: Vec<MaterialParameterBinding>,
    pub push_constants: Vec<u8>,
}

/// A PBR material builder. Holds typed, named parameters and assembles them into a flat `Material`.
pub struct PbrMaterial {
    pub vertex_shader: ShaderRef,
    pub fragment_shader: ShaderRef,
    pub base_color: MaterialColorParameter,
    pub normal: MaterialColorParameter,
    pub ambient_occlusion: MaterialParameter,
    pub metallic: MaterialParameter,
    pub roughness: MaterialParameter,
    pub specular: MaterialParameter,
}

impl PbrMaterial {
    /// Consumes the builder and produces the flat runtime `Material`.
    pub fn build(self) -> Material {
        let active_defines = self.compute_defines();
        let bindings = self.compute_bindings();
        let push_constants = self.compute_push_constants();
        Material {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            active_defines,
            bindings,
            push_constants,
        }
    }

    fn compute_defines(&self) -> Vec<String> {
        let mut d = Vec::new();
        if self.base_color.as_handle().is_some() {
            d.push("HAS_COLOR_TEXTURE".into());
        }
        if self.normal.as_handle().is_some() {
            d.push("HAS_NORMAL_TEXTURE".into());
        }
        if self.ambient_occlusion.as_handle().is_some()
            || self.metallic.as_handle().is_some()
            || self.roughness.as_handle().is_some()
            || self.specular.as_handle().is_some()
        {
            d.push("HAS_ORM_TEXTURE".into());
        }
        d
    }

    fn compute_bindings(&self) -> Vec<MaterialParameterBinding> {
        let base_color_binding = match self.base_color {
            MaterialColorParameter::Handle(handle) => Some(MaterialParameterBinding {
                index: 0,
                data: MaterialParameterBindingData::Texture(handle),
            }),
            _ => None,
        };

        let normal_binding = match self.normal {
            MaterialColorParameter::Handle(handle) => Some(MaterialParameterBinding {
                index: 1,
                data: MaterialParameterBindingData::Texture(handle),
            }),
            _ => None,
        };

        let packed_texture = if self.ambient_occlusion.as_handle().is_some()
            || self.metallic.as_handle().is_some()
            || self.roughness.as_handle().is_some()
            || self.specular.as_handle().is_some()
        {
            let packed = PackedTextureData {
                channel_r: self.ambient_occlusion.as_handle(),
                channel_g: self.metallic.as_handle(),
                channel_b: self.roughness.as_handle(),
                channel_a: self.specular.as_handle(),
            };
            Some(MaterialParameterBinding {
                index: 2,
                data: MaterialParameterBindingData::PackedTexture(packed),
            })
        } else {
            None
        };

        [base_color_binding, normal_binding, packed_texture]
            .into_iter()
            .flatten()
            .collect()
    }

    fn compute_push_constants(&self) -> Vec<u8> {
        let data = PbrPushConstants {
            base_color: self.base_color.as_constant(vec4(0.0, 0.0, 0.0, 0.0)),
            normal: self.normal.as_constant(vec4(0.0, 0.0, 0.0, 0.0)),
            roughness: self.roughness.as_constant(1.0),
            specular: self.specular.as_constant(0.0),
            metallic: self.metallic.as_constant(0.0),
            ambient_occlusion: self.ambient_occlusion.as_constant(0.0),
        };

        let bytes = unsafe {
            ::core::slice::from_raw_parts(
                (&data as *const PbrPushConstants) as *const u8,
                ::core::mem::size_of::<PbrPushConstants>(),
            )
        };

        bytes.to_vec()
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
    pub fn as_handle(&self) -> Option<ImageHandle> {
        match self {
            MaterialParameter::Handle(handle) => Some(*handle),
            _ => None,
        }
    }

    pub fn as_constant(&self, default: f32) -> f32 {
        match self {
            MaterialParameter::Constant(c) => *c,
            _ => default,
        }
    }
}

pub enum MaterialColorParameter {
    Handle(ImageHandle),
    Constant(Vec4),
}

impl MaterialColorParameter {
    pub fn as_handle(&self) -> Option<ImageHandle> {
        match self {
            MaterialColorParameter::Handle(handle) => Some(*handle),
            _ => None,
        }
    }

    pub fn as_constant(&self, default: Vec4) -> Vec4 {
        match self {
            MaterialColorParameter::Constant(c) => *c,
            _ => default,
        }
    }
}

#[derive(Clone)]
pub struct MaterialParameterBinding {
    pub index: usize,
    pub data: MaterialParameterBindingData,
}

#[derive(Clone)]
pub enum MaterialParameterBindingData {
    Texture(ImageHandle),
    /// Four individual texture handles to be channel-packed into a single RGBA texture at cook time
    /// by the material conditioner. The renderer receives a single packed handle after conditioning.
    PackedTexture(PackedTextureData),
}

#[derive(Clone)]
pub struct PackedTextureData {
    pub channel_r: Option<ImageHandle>,
    pub channel_g: Option<ImageHandle>,
    pub channel_b: Option<ImageHandle>,
    pub channel_a: Option<ImageHandle>,
}
