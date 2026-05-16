use common::Guid;
use serde::{Deserialize, Serialize};
use spirq::var::Variable;
use spirq::ReflectConfig;
use std::fmt;
use std::path::Path;
use std::process::Command;

pub struct ShaderPermutationDecl {
    pub name: String,
    pub requires: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ShaderDescriptor {
    pub set: u32,
    pub binding: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ShaderLayoutFile {
    pub bindings: Vec<ShaderDescriptor>,
}

impl ShaderLayoutFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ShaderLayoutError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ShaderLayoutError> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ShaderManifestFile {
    pub variants: Vec<ShaderVariant>,
}

/// Maps a set of defines to the compiled `.spv` filename for that variant.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ShaderVariant {
    pub defines: Vec<String>,
    pub file: String,
}

impl ShaderManifestFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ShaderLayoutError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ShaderLayoutError> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ShaderLayoutError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
}

impl fmt::Display for ShaderLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShaderLayoutError::Io(e) => write!(f, "io error: {}", e),
            ShaderLayoutError::Parse(e) => write!(f, "parse error: {}", e),
            ShaderLayoutError::Serialize(e) => write!(f, "serialize error: {}", e),
        }
    }
}

impl From<std::io::Error> for ShaderLayoutError {
    fn from(e: std::io::Error) -> Self {
        ShaderLayoutError::Io(e)
    }
}

impl From<toml::de::Error> for ShaderLayoutError {
    fn from(e: toml::de::Error) -> Self {
        ShaderLayoutError::Parse(e)
    }
}

impl From<toml::ser::Error> for ShaderLayoutError {
    fn from(e: toml::ser::Error) -> Self {
        ShaderLayoutError::Serialize(e)
    }
}

#[derive(Debug)]
pub enum ShaderConditionError {
    Io(std::io::Error),
    UnsupportedFormat(String),
    CompileError(String),
    Layout(ShaderLayoutError),
    InvalidSpirv,
}

impl fmt::Display for ShaderConditionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShaderConditionError::Io(e) => write!(f, "io: {}", e),
            ShaderConditionError::UnsupportedFormat(ext) => {
                write!(f, "unsupported shader format: {}", ext)
            }
            ShaderConditionError::CompileError(e) => write!(f, "failed to compile shader: {}", e),
            ShaderConditionError::InvalidSpirv => write!(f, "invalid spirv"),
            ShaderConditionError::Layout(e) => write!(f, "layout: {}", e),
        }
    }
}

impl From<std::io::Error> for ShaderConditionError {
    fn from(e: std::io::Error) -> Self {
        ShaderConditionError::Io(e)
    }
}

impl From<ShaderLayoutError> for ShaderConditionError {
    fn from(e: ShaderLayoutError) -> Self {
        ShaderConditionError::Layout(e)
    }
}

pub struct ShaderConditioner {}

impl ShaderConditioner {
    pub fn condition(
        src_path: &Path,
        guid: Guid,
        cooked_dir: &Path,
    ) -> Result<(), ShaderConditionError> {
        match src_path.extension().and_then(|e| e.to_str()) {
            Some("glsl" | "vert" | "frag" | "comp") => {}
            Some(ext) => return Err(ShaderConditionError::UnsupportedFormat(ext.to_string())),
            None => {
                return Err(ShaderConditionError::UnsupportedFormat(
                    "(none)".to_string(),
                ))
            }
        }

        let shader_contents = std::fs::read_to_string(src_path)?;
        let permutations = Self::scan_permutations(&shader_contents);
        let valid_permutations = Self::build_permutations(&permutations);

        let mut variants = Vec::new();

        for permutation in valid_permutations {
            let file_name = if permutation.is_empty() {
                format!("{}.spv", guid)
            } else {
                format!("{}.{}.spv", guid, permutation.join("."))
            };
            let spv_path = cooked_dir.join(&file_name);

            Self::compile_variant(src_path, &permutation, &spv_path)?;

            let bindings = Self::spirv_reflect(&spv_path)?;
            Self::create_layout_file(&spv_path.with_extension("layout"), bindings)?;

            variants.push(ShaderVariant {
                defines: permutation.iter().map(|s| s.to_string()).collect(),
                file: file_name,
            });
        }

        let manifest_path = cooked_dir.join(format!("{}.manifest", guid));
        Self::create_manifest_file(&manifest_path, variants)?;

        Ok(())
    }

    fn scan_permutations(src: &str) -> Vec<ShaderPermutationDecl> {
        src.lines()
            .filter_map(|l| l.trim().strip_prefix("#pragma permutation "))
            .map(|rest| {
                let mut parts = rest.split_whitespace();
                let name = parts.next().unwrap().to_string();
                let requires = if parts.next() == Some("requires") {
                    parts.next().map(str::to_string)
                } else {
                    None
                };
                ShaderPermutationDecl { name, requires }
            })
            .collect()
    }

    fn build_permutations(shader_permutation_decl: &[ShaderPermutationDecl]) -> Vec<Vec<&str>> {
        let names: Vec<&str> = shader_permutation_decl
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        let n = names.len();

        (0..(1u32 << n))
            .map(|mask| {
                names
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, name)| *name)
                    .collect::<Vec<_>>()
            })
            .filter(|combo| {
                shader_permutation_decl.iter().all(|d| {
                    let present = combo.contains(&d.name.as_str());
                    if let (true, Some(req)) = (present, &d.requires) {
                        combo.contains(&req.as_str())
                    } else {
                        true
                    }
                })
            })
            .collect()
    }

    fn compile_variant(
        src_path: &Path,
        defines: &[&str],
        dst_path: &Path,
    ) -> Result<(), ShaderConditionError> {
        let mut command = Command::new("glslc");
        command.arg(src_path).arg("-o").arg(dst_path);

        for define in defines {
            command.arg(format!("-D{}", define));
        }

        let output = command.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            return Err(ShaderConditionError::CompileError(stderr));
        }

        Ok(())
    }

    fn spirv_reflect(spv_path: &Path) -> Result<Vec<ShaderDescriptor>, ShaderConditionError> {
        let contents = std::fs::read(spv_path)?;

        let entry_point = ReflectConfig::new()
            .spv(contents)
            .reflect()
            .map_err(|_| ShaderConditionError::InvalidSpirv)
            .and_then(|v| {
                v.into_iter()
                    .next()
                    .ok_or(ShaderConditionError::InvalidSpirv)
            })?;

        let bindings = entry_point
            .vars
            .iter()
            .filter_map(|var| match var {
                Variable::Descriptor {
                    name,
                    desc_bind,
                    desc_ty,
                    ..
                } => Some(ShaderDescriptor {
                    set: desc_bind.set(),
                    binding: desc_bind.bind(),
                    type_name: format!("{:?}", desc_ty),
                    name: name.as_deref().unwrap_or("").to_string(),
                }),
                _ => None,
            })
            .collect();

        Ok(bindings)
    }

    fn create_layout_file(
        dst_path: &Path,
        bindings: Vec<ShaderDescriptor>,
    ) -> Result<(), ShaderConditionError> {
        ShaderLayoutFile { bindings }.save(dst_path)?;
        Ok(())
    }

    fn create_manifest_file(
        dst_path: &Path,
        variants: Vec<ShaderVariant>,
    ) -> Result<(), ShaderConditionError> {
        ShaderManifestFile { variants }.save(dst_path)?;
        Ok(())
    }
}