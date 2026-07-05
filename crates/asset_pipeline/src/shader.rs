#![allow(dead_code)]
use crate::emat::ParamValue;
use project::Guid;
use serde::Deserialize;
use std::fmt;
use std::path::Path;

#[derive(Deserialize)]
pub struct ShaderFile {
    pub bindings: Vec<ShaderBindings>,
    pub push_constants: Vec<ShaderPushConstants>,
}

#[derive(Deserialize)]
pub struct ShaderBindings {
    pub vert: Guid,
    pub frag: Guid,

    pub name: Option<String>,
    #[serde(rename = "type")]
    pub binding_type: Option<ParamValue>,
    pub binding_index: Option<u32>,
}

#[derive(Deserialize)]
pub struct ShaderPushConstants {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub push_constant_type: Option<ParamValue>,
    pub offset: Option<u32>,
}

#[derive(Debug)]
pub enum ShaderError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    MissingShader,
    /// A texture GUID could not be resolved in the registry.
    UnresolvedGuid(Guid),
    /// The asset manager failed to load the image for a texture param.
    ImageLoadFailed(String),
    /// Custom shader manifest loading is not yet implemented.
    NotYetImplemented(String),
}

impl fmt::Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShaderError::Io(e) => write!(f, "io: {}", e),
            ShaderError::Parse(e) => write!(f, "parse: {}", e),
            ShaderError::UnresolvedGuid(g) => write!(f, "unresolved guid '{}'", g),
            ShaderError::ImageLoadFailed(p) => write!(f, "failed to load image at '{}'", p),
            ShaderError::NotYetImplemented(s) => write!(f, "not yet implemented: {}", s),
            ShaderError::MissingShader => write!(f, "shader missing"),
        }
    }
}

impl From<std::io::Error> for ShaderError {
    fn from(e: std::io::Error) -> Self {
        ShaderError::Io(e)
    }
}
impl From<toml::de::Error> for ShaderError {
    fn from(e: toml::de::Error) -> Self {
        ShaderError::Parse(e)
    }
}

impl ShaderFile {
    pub fn load(path: &Path) -> Result<Self, ShaderError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
