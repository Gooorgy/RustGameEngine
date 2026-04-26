use crate::Guid;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Sidecar file stored alongside each source asset (e.g. `wall.png.meta`).
/// The source of truth for asset identity and import settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetMeta {
    pub guid: Guid,
    /// Conditioner-specific import settings. Interpreted by the relevant
    /// conditioner at cook time; the project crate does not inspect them.
    #[serde(default, skip_serializing_if = "table_is_empty")]
    pub import: toml::Table,
}

#[derive(Debug)]
pub enum MetaError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
}

impl fmt::Display for MetaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaError::Io(e) => write!(f, "io error: {}", e),
            MetaError::Parse(e) => write!(f, "parse error: {}", e),
            MetaError::Serialize(e) => write!(f, "serialize error: {}", e),
        }
    }
}

use std::fmt;

impl From<std::io::Error> for MetaError {
    fn from(e: std::io::Error) -> Self {
        MetaError::Io(e)
    }
}

impl From<toml::de::Error> for MetaError {
    fn from(e: toml::de::Error) -> Self {
        MetaError::Parse(e)
    }
}

impl From<toml::ser::Error> for MetaError {
    fn from(e: toml::ser::Error) -> Self {
        MetaError::Serialize(e)
    }
}

impl AssetMeta {
    /// Loads the `.meta` sidecar for `source_path`, creating one with a fresh
    /// GUID if it does not exist yet.
    pub fn load_or_create(source_path: impl AsRef<Path>) -> Result<Self, MetaError> {
        let meta_path = Self::meta_path_for(source_path.as_ref());
        if meta_path.exists() {
            Self::load(&meta_path)
        } else {
            let meta = Self {
                guid: Guid::generate(),
                import: toml::Table::new(),
            };
            meta.save(&meta_path)?;
            Ok(meta)
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, MetaError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), MetaError> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Returns the `.meta` sidecar path for a given source asset path.
    /// `wall.png` → `wall.png.meta`, `cube.obj` → `cube.obj.meta`
    pub fn meta_path_for(source_path: &Path) -> PathBuf {
        let ext = source_path
            .extension()
            .map(|e| format!("{}.meta", e.to_string_lossy()))
            .unwrap_or_else(|| "meta".to_string());
        source_path.with_extension(ext)
    }
}

fn table_is_empty(t: &toml::Table) -> bool {
    t.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_path_for_appends_meta() {
        assert_eq!(
            AssetMeta::meta_path_for(Path::new("wall.png")),
            PathBuf::from("wall.png.meta")
        );
        assert_eq!(
            AssetMeta::meta_path_for(Path::new("cube.obj")),
            PathBuf::from("cube.obj.meta")
        );
        assert_eq!(
            AssetMeta::meta_path_for(Path::new("water.shader")),
            PathBuf::from("water.shader.meta")
        );
    }

    #[test]
    fn round_trip_serialization() {
        let meta = AssetMeta {
            guid: Guid::generate(),
            import: toml::Table::new(),
        };
        let serialized = toml::to_string_pretty(&meta).unwrap();
        let deserialized: AssetMeta = toml::from_str(&serialized).unwrap();
        assert_eq!(meta.guid, deserialized.guid);
    }

    #[test]
    fn load_or_create_generates_stable_guid() {
        let dir = std::env::temp_dir().join(format!("eproj_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("test.png");
        std::fs::write(&source, b"").unwrap();

        let meta1 = AssetMeta::load_or_create(&source).unwrap();
        let meta2 = AssetMeta::load_or_create(&source).unwrap();
        assert_eq!(meta1.guid, meta2.guid);

        // cleanup
        std::fs::remove_dir_all(&dir).unwrap();
    }
}