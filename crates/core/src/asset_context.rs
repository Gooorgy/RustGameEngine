use asset_pipeline::EmatFile;
use assets::AssetStore;
use common::MeshHandle;
use material::Material;
use project::{AssetRegistry, Guid, Project};
use std::path::{Path, PathBuf};

/// Owns the project, asset registry, and runtime CPU-side asset caches.
/// All asset loading that needs to survive into level serialization goes through here.
/// New raw asset types (meshes, textures, shaders) add methods here.
pub struct AssetContext {
    pub(crate) project: Project,
    pub(crate) registry: AssetRegistry,
    pub(crate) asset_store: AssetStore,
}

impl AssetContext {
    pub fn new(project: Project, registry: AssetRegistry) -> Self {
        Self {
            project,
            registry,
            asset_store: AssetStore::new(),
        }
    }

    /// Development convenience: resolves a source path to its GUID via the
    /// registry, then loads the cooked `.emesh` from the cache.
    ///
    /// This exists because game code currently has no editor to assign assets
    /// by GUID. Once a level system exists, assets will be referenced by GUID
    /// directly in level files and this path-based API will no longer be needed.
    pub fn load_mesh<P: AsRef<Path>>(&mut self, path: P) -> MeshHandle {
        let path = path.as_ref();
        let rel = self.to_rel(path);

        let record = self.registry.find_by_source_path(&rel).unwrap_or_else(|| {
            panic!(
                "mesh not in registry: {} (rel: {})",
                path.display(),
                rel.display()
            )
        });

        let guid = record.guid;
        let cooked = self.project.cooked_path(&guid, "emesh");
        self.asset_store
            .load_mesh(&cooked, guid)
            .unwrap_or_else(|| {
                panic!(
                    "cooked mesh missing for '{}' — was cook_pending called at startup? (expected: {})",
                    rel.display(),
                    cooked.display()
                )
            })
    }

    /// Development convenience: resolves a source path to its GUID, builds the
    /// material from the `.emat` file, and returns the built material with its GUID.
    ///
    /// This exists because game code currently has no editor to assign materials
    /// by GUID. Once a level system exists, materials will be referenced by GUID
    /// directly in level files and this path-based API will no longer be needed.
    pub fn build_material<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> (Box<dyn Material>, Option<Guid>) {
        let path = path.as_ref();
        let rel = self.to_rel(path);
        let guid = self.registry.find_by_source_path(&rel).map(|r| r.guid);

        let mat = EmatFile::load(path)
            .and_then(|f| f.build_material(&self.project, &self.registry, &mut self.asset_store))
            .unwrap_or_else(|e| panic!("failed to load material '{}': {}", path.display(), e));

        (mat, guid)
    }

    pub(crate) fn store(&self) -> &AssetStore {
        &self.asset_store
    }

    // ── Path helpers ───────────────────────────────────────────────────────

    fn to_rel(&self, path: &Path) -> PathBuf {
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.project.root.join(path)
        };
        abs.strip_prefix(&self.project.content_dir)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path.to_path_buf())
    }
}