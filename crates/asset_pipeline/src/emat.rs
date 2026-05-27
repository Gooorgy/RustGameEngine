use assets::AssetStore;
use common::{Guid, ImageHandle};
use material::{
    Material, MaterialColorParameter, MaterialParameter, MaterialParameterBinding,
    MaterialParameterBindingData, PbrMaterial, ShaderRef,
};
use nalgebra_glm::{vec4, Vec4};
use project::{AssetRegistry, Project};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

// ── File format types ──────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct EmatFile {
    /// "pbr" for the built-in PBR material path. Omit for fully custom materials.
    #[serde(rename = "type")]
    pub mat_type: Option<String>,
    /// Vertex shader: a UUID string resolves to `ShaderRef::Asset`, a plain name to `ShaderRef::BuiltIn`.
    /// Defaults to the engine built-in `"vert"` when absent.
    pub vertex_shader: Option<String>,
    /// Fragment shader: a UUID string resolves to `ShaderRef::Asset`, a plain name to `ShaderRef::BuiltIn`.
    /// Required when `type` is absent (custom material). Defaults to built-in PBR for `type = "pbr"`.
    pub fragment_shader: Option<String>,
    #[serde(default)]
    pub params: HashMap<String, ParamValue>,
}

/// A single named material parameter -- either a texture reference or an inline constant.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    Texture { texture: Guid },
    Vec4 { value: [f32; 4] },
    Float { value: f32 },
}

// ── Errors ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum EmatError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    /// The `type` field contained an unrecognised value.
    UnknownType(String),
    /// The file has neither `type` nor `fragment_shader` set.
    MissingTypeOrShader,
    /// A GUID could not be resolved in the registry.
    UnresolvedGuid(Guid),
    /// The asset manager failed to load the image for a texture param.
    ImageLoadFailed(Guid),
}

impl fmt::Display for EmatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmatError::Io(e) => write!(f, "io: {}", e),
            EmatError::Parse(e) => write!(f, "parse: {}", e),
            EmatError::UnknownType(t) => write!(f, "unknown material type '{}'", t),
            EmatError::MissingTypeOrShader => {
                write!(f, "emat must have 'type' or 'fragment_shader'")
            }
            EmatError::UnresolvedGuid(g) => write!(f, "unresolved guid '{}'", g),
            EmatError::ImageLoadFailed(g) => write!(f, "failed to load image for guid '{}'", g),
        }
    }
}

impl From<std::io::Error> for EmatError {
    fn from(e: std::io::Error) -> Self {
        EmatError::Io(e)
    }
}

impl From<toml::de::Error> for EmatError {
    fn from(e: toml::de::Error) -> Self {
        EmatError::Parse(e)
    }
}

// ── EmatFile methods ───────────────────────────────────────────────────────

impl EmatFile {
    /// Reads and parses a `.emat` file. Does no asset resolution.
    pub fn load(path: &Path) -> Result<Self, EmatError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Resolves all asset references and returns a `Material` ready for
    /// registration with `MaterialManager::add_material_instance`.
    pub fn build_material(
        &self,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetStore,
    ) -> Result<Material, EmatError> {
        match self.mat_type.as_deref() {
            Some("pbr") => self.build_pbr(project, registry, assets),
            Some(other) => Err(EmatError::UnknownType(other.to_string())),
            None => {
                self.fragment_shader
                    .as_deref()
                    .ok_or(EmatError::MissingTypeOrShader)?;
                self.build_custom(project, registry, assets)
            }
        }
    }

    fn build_pbr(
        &self,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetStore,
    ) -> Result<Material, EmatError> {
        Ok(PbrMaterial {
            vertex_shader: parse_shader_ref(self.vertex_shader.as_deref(), "vert"),
            fragment_shader: parse_shader_ref(self.fragment_shader.as_deref(), "pbr.frag"),
            base_color: self.color_param("base_color", project, registry, assets)?,
            normal: self.color_param("normal", project, registry, assets)?,
            ambient_occlusion: self.scalar_param("ambient_occlusion", project, registry, assets)?,
            metallic: self.scalar_param("metallic", project, registry, assets)?,
            roughness: self.scalar_param("roughness", project, registry, assets)?,
            specular: self.scalar_param("specular", project, registry, assets)?,
        }
        .build())
    }

    fn build_custom(
        &self,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetStore,
    ) -> Result<Material, EmatError> {
        let fragment_shader = parse_shader_ref(self.fragment_shader.as_deref(), "");
        let vertex_shader = parse_shader_ref(self.vertex_shader.as_deref(), "vert");

        let mut sorted_keys: Vec<&String> = self.params.keys().collect();
        sorted_keys.sort();

        let mut bindings = Vec::new();
        for (index, key) in sorted_keys.iter().enumerate() {
            if let Some(ParamValue::Texture { texture }) = self.params.get(*key) {
                let handle = self.resolve_texture(texture, project, registry, assets)?;
                bindings.push(MaterialParameterBinding {
                    index,
                    data: MaterialParameterBindingData::Texture(handle),
                });
            }
        }

        Ok(Material {
            vertex_shader,
            fragment_shader,
            active_defines: Vec::new(),
            bindings,
            push_constants: Vec::new(),
        })
    }

    fn color_param(
        &self,
        name: &str,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetStore,
    ) -> Result<MaterialColorParameter, EmatError> {
        match self.params.get(name) {
            Some(ParamValue::Texture { texture }) => {
                let handle = self.resolve_texture(texture, project, registry, assets)?;
                Ok(MaterialColorParameter::Handle(handle))
            }
            Some(ParamValue::Vec4 { value: v }) => {
                Ok(MaterialColorParameter::Constant(vec4(v[0], v[1], v[2], v[3])))
            }
            Some(ParamValue::Float { value: v }) => {
                Ok(MaterialColorParameter::Constant(vec4(*v, *v, *v, 1.0)))
            }
            None => Ok(MaterialColorParameter::Constant(Vec4::zeros())),
        }
    }

    fn scalar_param(
        &self,
        name: &str,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetStore,
    ) -> Result<MaterialParameter, EmatError> {
        match self.params.get(name) {
            Some(ParamValue::Texture { texture }) => {
                let handle = self.resolve_texture(texture, project, registry, assets)?;
                Ok(MaterialParameter::Handle(handle))
            }
            Some(ParamValue::Float { value: v }) => Ok(MaterialParameter::Constant(*v)),
            Some(ParamValue::Vec4 { .. }) => Err(EmatError::UnknownType(format!(
                "param '{}': vec4 cannot be used for a scalar slot",
                name
            ))),
            None => Ok(MaterialParameter::Constant(0.0)),
        }
    }

    fn resolve_texture(
        &self,
        guid: &Guid,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetStore,
    ) -> Result<ImageHandle, EmatError> {
        registry.get(guid).ok_or(EmatError::UnresolvedGuid(*guid))?;
        let cooked_path = project.cooked_path(guid, "etex");
        assets
            .load_texture(&cooked_path, *guid)
            .ok_or(EmatError::ImageLoadFailed(*guid))
    }
}

/// Parses a shader string into a `ShaderRef`.
/// A valid UUID resolves to `ShaderRef::Asset`; anything else is treated as a built-in name.
/// If `s` is `None`, returns `ShaderRef::BuiltIn(default_builtin)`.
fn parse_shader_ref(s: Option<&str>, default_builtin: &str) -> ShaderRef {
    match s {
        None => ShaderRef::BuiltIn(default_builtin.into()),
        Some(s) => match Guid::from_str(s) {
            Some(guid) => ShaderRef::Asset(guid),
            None => ShaderRef::BuiltIn(s.into()),
        },
    }
}
