use crate::Guid;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
struct ProjectFile {
    name: String,
    engine_version: String,
    content_dir: String,
}

/// Loaded representation of a `.eproj` file.
pub struct Project {
    pub name: String,
    pub engine_version: String,
    /// Directory containing the `.eproj` file.
    pub root: PathBuf,
    /// Resolved absolute path to the content directory.
    pub content_dir: PathBuf,
    /// Resolved absolute path to the cooked asset cache.
    pub cache_dir: PathBuf,
}

#[derive(Debug)]
pub enum ProjectError {
    Io(std::io::Error),
    Parse(toml::de::Error),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::Io(e) => write!(f, "io error: {}", e),
            ProjectError::Parse(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl From<std::io::Error> for ProjectError {
    fn from(e: std::io::Error) -> Self {
        ProjectError::Io(e)
    }
}

impl From<toml::de::Error> for ProjectError {
    fn from(e: toml::de::Error) -> Self {
        ProjectError::Parse(e)
    }
}

impl Project {
    /// Loads a project from a `.eproj` file. Paths inside the project file are
    /// resolved relative to the directory containing the `.eproj`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let path = path.as_ref();
        let root = path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf()
            .canonicalize()?;

        let content = std::fs::read_to_string(path)?;
        let file: ProjectFile = toml::from_str(&content)?;

        let content_dir = root.join(&file.content_dir);
        let cache_dir = root.join(".cache");

        Ok(Self {
            name: file.name,
            engine_version: file.engine_version,
            root,
            content_dir,
            cache_dir,
        })
    }

    /// Returns the expected cooked output path for an asset with the given GUID
    /// and file extension (e.g. `"etex"`, `"emesh"`).
    pub fn cooked_path(&self, guid: &Guid, extension: &str) -> PathBuf {
        self.cache_dir
            .join("cooked")
            .join(format!("{}.{}", guid, extension))
    }
}