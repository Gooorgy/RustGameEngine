use assets::{AssetManager, ImageHandle};
use material::{Material, MaterialColorParameter, MaterialParameter, PbrMaterial};
use nalgebra_glm::{vec4, Vec4};
use project::{AssetRegistry, Guid, Project};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

// ── File format types ──────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct EmatFile {
    /// "pbr" for the built-in PBR material path.
    #[serde(rename = "type")]
    pub mat_type: Option<String>,
    /// GUID string referencing a `.shader` manifest for custom shader materials.
    pub shader: Option<String>,
    #[serde(default)]
    pub params: HashMap<String, ParamValue>,
}

/// A single named material parameter -- either a texture reference or an
/// inline constant.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    Texture { texture: String },
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
    /// The file has neither `type` nor `shader` set.
    MissingTypeOrShader,
    /// A texture GUID could not be resolved in the registry.
    UnresolvedGuid(String),
    /// The asset manager failed to load the image for a texture param.
    ImageLoadFailed(String),
    /// Custom shader manifest loading is not yet implemented.
    NotYetImplemented(String),
}

impl fmt::Display for EmatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmatError::Io(e) => write!(f, "io: {}", e),
            EmatError::Parse(e) => write!(f, "parse: {}", e),
            EmatError::UnknownType(t) => write!(f, "unknown material type '{}'", t),
            EmatError::MissingTypeOrShader => write!(f, "emat must have 'type' or 'shader'"),
            EmatError::UnresolvedGuid(g) => write!(f, "unresolved guid '{}'", g),
            EmatError::ImageLoadFailed(p) => write!(f, "failed to load image at '{}'", p),
            EmatError::NotYetImplemented(s) => write!(f, "not yet implemented: {}", s),
        }
    }
}

impl From<std::io::Error> for EmatError {
    fn from(e: std::io::Error) -> Self { EmatError::Io(e) }
}
impl From<toml::de::Error> for EmatError {
    fn from(e: toml::de::Error) -> Self { EmatError::Parse(e) }
}

// ── EmatFile methods ───────────────────────────────────────────────────────

impl EmatFile {
    /// Reads and parses a `.emat` file. Does no asset resolution.
    pub fn load(path: &Path) -> Result<Self, EmatError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Resolves all asset references and returns a boxed `Material` ready for
    /// registration with `MaterialManager::add_material_instance`.
    pub fn build_material(
        &self,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetManager,
    ) -> Result<Box<dyn Material>, EmatError> {
        match (self.mat_type.as_deref(), self.shader.as_deref()) {
            (Some("pbr"), _) => {
                let pbr = self.build_pbr(project, registry, assets)?;
                Ok(Box::new(pbr))
            }
            (Some(other), _) => Err(EmatError::UnknownType(other.to_string())),
            (None, Some(_shader_guid)) => {
                Err(EmatError::NotYetImplemented(
                    "custom shader materials (.shader manifest loading) are not yet implemented".to_string(),
                ))
            }
            (None, None) => Err(EmatError::MissingTypeOrShader),
        }
    }

    fn build_pbr(
        &self,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetManager,
    ) -> Result<PbrMaterial, EmatError> {
        Ok(PbrMaterial {
            base_color: self.color_param("base_color", project, registry, assets)?,
            normal: self.color_param("normal", project, registry, assets)?,
            ambient_occlusion: self.scalar_param("ambient_occlusion", project, registry, assets)?,
            metallic: self.scalar_param("metallic", project, registry, assets)?,
            roughness: self.scalar_param("roughness", project, registry, assets)?,
            specular: self.scalar_param("specular", project, registry, assets)?,
        })
    }

    fn color_param(
        &self,
        name: &str,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetManager,
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
        assets: &mut AssetManager,
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
        guid_str: &str,
        project: &Project,
        registry: &AssetRegistry,
        assets: &mut AssetManager,
    ) -> Result<ImageHandle, EmatError> {
        let guid = Guid::from_str(guid_str)
            .ok_or_else(|| EmatError::UnresolvedGuid(guid_str.to_string()))?;

        let record = registry
            .get(&guid)
            .ok_or_else(|| EmatError::UnresolvedGuid(guid_str.to_string()))?;

        let abs_path = project.content_dir.join(&record.source_path);

        assets
            .get_image(&abs_path)
            .ok_or_else(|| EmatError::ImageLoadFailed(abs_path.to_string_lossy().into_owned()))
    }
}