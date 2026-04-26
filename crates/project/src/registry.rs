use crate::{AssetMeta, Guid, Project};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Index of every known asset in the project content directory.
pub struct AssetRegistry {
    records: HashMap<Guid, AssetRecord>,
}

pub struct AssetRecord {
    pub guid: Guid,
    /// Path relative to the project content directory.
    pub source_path: PathBuf,
    pub asset_type: AssetType,
    /// FNV-1a hash of the source file bytes at last scan.
    pub source_hash: u64,
    /// FNV-1a hash of the import settings at last scan.
    pub import_hash: u64,
    pub status: AssetStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Shader,
    ShaderManifest,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetStatus {
    /// Cooked output exists and source hashes match.
    Fresh,
    /// Source or import settings changed since the asset was last cooked.
    Dirty,
    /// Asset has never been cooked.
    Uncooked,
}

// ── Serialized form stored in .cache/.assetdb ──────────────────────────────

#[derive(Serialize, Deserialize)]
struct RegistryFile {
    #[serde(default)]
    records: HashMap<String, RecordFile>,
}

#[derive(Serialize, Deserialize)]
struct RecordFile {
    source_path: String,
    asset_type: AssetType,
    // u64 serialized as hex to avoid TOML's signed i64 ceiling
    source_hash: String,
    import_hash: String,
}

// ── Errors ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum RegistryError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
    Meta(crate::MetaError),
    Walk(walkdir::Error),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::Io(e) => write!(f, "io: {}", e),
            RegistryError::Parse(e) => write!(f, "parse: {}", e),
            RegistryError::Serialize(e) => write!(f, "serialize: {}", e),
            RegistryError::Meta(e) => write!(f, "meta: {}", e),
            RegistryError::Walk(e) => write!(f, "walk: {}", e),
        }
    }
}

impl From<std::io::Error> for RegistryError {
    fn from(e: std::io::Error) -> Self { RegistryError::Io(e) }
}
impl From<toml::de::Error> for RegistryError {
    fn from(e: toml::de::Error) -> Self { RegistryError::Parse(e) }
}
impl From<toml::ser::Error> for RegistryError {
    fn from(e: toml::ser::Error) -> Self { RegistryError::Serialize(e) }
}
impl From<crate::MetaError> for RegistryError {
    fn from(e: crate::MetaError) -> Self { RegistryError::Meta(e) }
}
impl From<walkdir::Error> for RegistryError {
    fn from(e: walkdir::Error) -> Self { RegistryError::Walk(e) }
}

// ── Implementation ─────────────────────────────────────────────────────────

impl AssetRegistry {
    /// Walks the content directory, creating `.meta` sidecars for any new
    /// asset files, and builds the full registry index.
    ///
    /// Pass a previously loaded registry as `previous` to detect dirty assets
    /// whose source has changed since the last scan.
    pub fn scan(
        project: &Project,
        previous: Option<&AssetRegistry>,
    ) -> Result<Self, RegistryError> {
        let mut records = HashMap::new();

        for entry in walkdir::WalkDir::new(&project.content_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if is_meta_file(path) {
                continue;
            }

            let meta = AssetMeta::load_or_create(path)?;
            let asset_type = AssetType::from_extension(
                path.extension().and_then(|e| e.to_str()).unwrap_or(""),
            );
            let source_hash = hash_file(path).unwrap_or(0);
            let import_hash = hash_table(&meta.import);

            let source_path = path
                .strip_prefix(&project.content_dir)
                .unwrap_or(path)
                .to_path_buf();

            let status = resolve_status(
                project,
                &meta.guid,
                asset_type,
                source_hash,
                import_hash,
                previous,
            );

            records.insert(
                meta.guid,
                AssetRecord {
                    guid: meta.guid,
                    source_path,
                    asset_type,
                    source_hash,
                    import_hash,
                    status,
                },
            );
        }

        Ok(Self { records })
    }

    /// Loads a previously saved registry from `.cache/.assetdb`, restoring
    /// status based on whether cooked outputs still exist on disk.
    /// Falls back to a full scan if the file is absent or unparseable.
    pub fn load_or_scan(project: &Project) -> Result<Self, RegistryError> {
        let db_path = project.cache_dir.join(".assetdb");
        if db_path.exists() {
            if let Ok(registry) = Self::load(&db_path, project) {
                return Ok(registry);
            }
        }
        Self::scan(project, None)
    }

    /// Saves the registry index to `.cache/.assetdb`.
    pub fn save(&self, project: &Project) -> Result<(), RegistryError> {
        std::fs::create_dir_all(&project.cache_dir)?;
        let db_path = project.cache_dir.join(".assetdb");

        let file = RegistryFile {
            records: self
                .records
                .iter()
                .map(|(guid, rec)| {
                    (
                        guid.to_string(),
                        RecordFile {
                            source_path: rec.source_path.to_string_lossy().into_owned(),
                            asset_type: rec.asset_type,
                            source_hash: format!("{:#018x}", rec.source_hash),
                            import_hash: format!("{:#018x}", rec.import_hash),
                        },
                    )
                })
                .collect(),
        };

        std::fs::write(db_path, toml::to_string_pretty(&file)?)?;
        Ok(())
    }

    pub fn get(&self, guid: &Guid) -> Option<&AssetRecord> {
        self.records.get(guid)
    }

    /// Assets that are `Dirty` or `Uncooked` — need to be (re-)cooked.
    pub fn pending(&self) -> impl Iterator<Item = &AssetRecord> {
        self.records
            .values()
            .filter(|r| r.status != AssetStatus::Fresh)
    }

    pub fn all(&self) -> impl Iterator<Item = &AssetRecord> {
        self.records.values()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    fn load(path: &Path, project: &Project) -> Result<Self, RegistryError> {
        let content = std::fs::read_to_string(path)?;
        let file: RegistryFile = toml::from_str(&content)?;

        let records = file
            .records
            .into_iter()
            .filter_map(|(guid_str, rec)| {
                let guid = Guid::from_str(&guid_str)?;
                let status = cooked_status(project, &guid, rec.asset_type);
                let source_hash = parse_hash(&rec.source_hash);
                let import_hash = parse_hash(&rec.import_hash);
                Some((
                    guid,
                    AssetRecord {
                        guid,
                        source_path: PathBuf::from(rec.source_path),
                        asset_type: rec.asset_type,
                        source_hash,
                        import_hash,
                        status,
                    },
                ))
            })
            .collect();

        Ok(Self { records })
    }
}

impl AssetType {
    fn from_extension(ext: &str) -> Self {
        match ext {
            "gltf" | "glb" | "obj" => AssetType::Mesh,
            "png" | "jpg" | "jpeg" | "hdr" | "exr" => AssetType::Texture,
            "emat" => AssetType::Material,
            "glsl" | "vert" | "frag" | "comp" => AssetType::Shader,
            "shader" => AssetType::ShaderManifest,
            _ => AssetType::Unknown,
        }
    }

    /// File extension used for this asset's cooked output, if any.
    pub fn cooked_extension(self) -> Option<&'static str> {
        match self {
            AssetType::Mesh => Some("emesh"),
            AssetType::Texture => Some("etex"),
            AssetType::Shader => Some("spv"),
            AssetType::Material | AssetType::ShaderManifest | AssetType::Unknown => None,
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn resolve_status(
    project: &Project,
    guid: &Guid,
    asset_type: AssetType,
    source_hash: u64,
    import_hash: u64,
    previous: Option<&AssetRegistry>,
) -> AssetStatus {
    let Some(ext) = asset_type.cooked_extension() else {
        return AssetStatus::Fresh; // not a cooked asset type
    };

    let cooked_exists = project.cooked_path(guid, ext).exists();

    if let Some(prev) = previous.and_then(|r| r.records.get(guid)) {
        if source_hash != prev.source_hash || import_hash != prev.import_hash {
            return if cooked_exists {
                AssetStatus::Dirty
            } else {
                AssetStatus::Uncooked
            };
        }
    }

    if cooked_exists {
        AssetStatus::Fresh
    } else {
        AssetStatus::Uncooked
    }
}

/// Status used when loading from `.assetdb` — trusts stored hashes, only
/// checks whether cooked output still exists on disk.
fn cooked_status(project: &Project, guid: &Guid, asset_type: AssetType) -> AssetStatus {
    let Some(ext) = asset_type.cooked_extension() else {
        return AssetStatus::Fresh;
    };
    if project.cooked_path(guid, ext).exists() {
        AssetStatus::Fresh
    } else {
        AssetStatus::Uncooked
    }
}

fn parse_hash(s: &str) -> u64 {
    let hex = s.trim_start_matches("0x");
    u64::from_str_radix(hex, 16).unwrap_or(0)
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_meta_file(path: &Path) -> bool {
    path.to_string_lossy().ends_with(".meta")
}

fn hash_file(path: &Path) -> std::io::Result<u64> {
    std::fs::read(path).map(|b| fnv1a(&b))
}

fn hash_table(table: &toml::Table) -> u64 {
    fnv1a(toml::to_string(table).unwrap_or_default().as_bytes())
}

fn fnv1a(data: &[u8]) -> u64 {
    const OFFSET: u64 = 14695981039346656037;
    const PRIME: u64 = 1099511628211;
    data.iter()
        .fold(OFFSET, |acc, &b| (acc ^ b as u64).wrapping_mul(PRIME))
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("png"), AssetType::Texture);
        assert_eq!(AssetType::from_extension("glb"), AssetType::Mesh);
        assert_eq!(AssetType::from_extension("obj"), AssetType::Mesh);
        assert_eq!(AssetType::from_extension("emat"), AssetType::Material);
        assert_eq!(AssetType::from_extension("glsl"), AssetType::Shader);
        assert_eq!(AssetType::from_extension("shader"), AssetType::ShaderManifest);
        assert_eq!(AssetType::from_extension("xyz"), AssetType::Unknown);
    }

    #[test]
    fn cooked_extension_mapping() {
        assert_eq!(AssetType::Mesh.cooked_extension(), Some("emesh"));
        assert_eq!(AssetType::Texture.cooked_extension(), Some("etex"));
        assert_eq!(AssetType::Shader.cooked_extension(), Some("spv"));
        assert_eq!(AssetType::Material.cooked_extension(), None);
    }

    #[test]
    fn scan_discovers_assets_and_creates_meta_files() {
        let dir = std::env::temp_dir().join(format!("reg_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("textures")).unwrap();
        std::fs::create_dir_all(dir.join("models")).unwrap();
        std::fs::write(dir.join("textures/wall.png"), b"fake png").unwrap();
        std::fs::write(dir.join("models/cube.obj"), b"fake obj").unwrap();

        let project = Project {
            name: "test".into(),
            engine_version: "0.1.0".into(),
            root: dir.clone(),
            content_dir: dir.clone(),
            cache_dir: dir.join(".cache"),
        };

        let registry = AssetRegistry::scan(&project, None).unwrap();
        assert_eq!(registry.len(), 2);

        // .meta sidecars should have been created
        assert!(AssetMeta::meta_path_for(&dir.join("textures/wall.png")).exists());
        assert!(AssetMeta::meta_path_for(&dir.join("models/cube.obj")).exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!("reg_rt_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("tex.png"), b"px").unwrap();

        let project = Project {
            name: "test".into(),
            engine_version: "0.1.0".into(),
            root: dir.clone(),
            content_dir: dir.clone(),
            cache_dir: dir.join(".cache"),
        };

        let registry = AssetRegistry::scan(&project, None).unwrap();
        registry.save(&project).unwrap();

        let loaded = AssetRegistry::load_or_scan(&project).unwrap();
        assert_eq!(loaded.len(), registry.len());

        let original_guid = registry.all().next().unwrap().guid;
        assert!(loaded.get(&original_guid).is_some());

        std::fs::remove_dir_all(&dir).unwrap();
    }
}