use asset_pipeline::EmatFile;
use assets::AssetStore;
use common::{Guid, MeshHandle};
use material::Material;
use project::{resolve_cooked_path, AssetRegistry};
use std::path::{Path, PathBuf};

pub struct AssetContext {
    pub(crate) cache_dir: PathBuf,
    pub(crate) content_dir: PathBuf,
    pub(crate) registry: AssetRegistry,
    pub(crate) asset_store: AssetStore,
}

impl AssetContext {
    pub fn new(cache_dir: PathBuf, content_dir: PathBuf, registry: AssetRegistry) -> Self {
        Self {
            cache_dir,
            content_dir,
            registry,
            asset_store: AssetStore::new(),
        }
    }

    pub fn load_mesh(&mut self, guid: Guid) -> MeshHandle {
        let cooked = resolve_cooked_path(&self.cache_dir, &guid, "emesh");
        self.asset_store
            .load_mesh(&cooked, guid)
            .unwrap_or_else(|| {
                panic!(
                    "cooked mesh missing for '{}' (expected: {})",
                    guid,
                    cooked.display()
                )
            })
    }

    /// Builds a `Material` from the `.emat` source file for the given GUID.
    /// Looks up the source path in the registry, then loads and builds the material.
    pub fn build_material(&mut self, guid: Guid) -> Material {
        let record = self
            .registry
            .get(&guid)
            .unwrap_or_else(|| panic!("no asset record for guid '{}'", guid));

        let abs = self.content_dir.join(&record.source_path);

        EmatFile::load(&abs)
            .and_then(|f| f.build_material(&self.cache_dir, &self.registry, &mut self.asset_store))
            .unwrap_or_else(|e| panic!("failed to load material '{}': {}", abs.display(), e))
    }

    pub(crate) fn store(&self) -> &AssetStore {
        &self.asset_store
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}