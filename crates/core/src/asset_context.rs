use asset_pipeline::EmatFile;
use assets::{AssetManager, MeshHandle};
use material::material_manager::{MaterialHandle, MaterialManager};
use project::{AssetRegistry, Guid, Project};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Owns the project, asset registry, runtime caches, and stable GUID mappings.
/// All asset loading that needs to survive into level serialization goes through here.
/// New asset types add methods here rather than on `EngineContext`.
pub struct AssetContext {
    pub(crate) project: Project,
    pub(crate) registry: AssetRegistry,
    pub(crate) asset_manager: Rc<RefCell<AssetManager>>,
    pub(crate) material_manager: Rc<RefCell<MaterialManager>>,
    // Stable GUID <-> runtime handle mappings for level serialization
    guid_to_mesh: HashMap<Guid, MeshHandle>,
    mesh_to_guid: HashMap<MeshHandle, Guid>,
    guid_to_material: HashMap<Guid, MaterialHandle>,
    material_to_guid: HashMap<MaterialHandle, Guid>,
}

impl AssetContext {
    pub fn new(project: Project, registry: AssetRegistry) -> Self {
        Self {
            project,
            registry,
            asset_manager: Rc::new(RefCell::new(AssetManager::default())),
            material_manager: Rc::new(RefCell::new(MaterialManager::new())),
            guid_to_mesh: HashMap::new(),
            mesh_to_guid: HashMap::new(),
            guid_to_material: HashMap::new(),
            material_to_guid: HashMap::new(),
        }
    }

    /// Loads a cooked `.emesh` for the given source path. The asset must have
    /// been cooked by `cook_pending` during `App::new()` before calling this.
    /// The handle is stable across sessions via its GUID.
    pub fn load_mesh<P: AsRef<Path>>(&mut self, path: P) -> MeshHandle {
        let path = path.as_ref();
        let rel = self.to_rel(path);

        let record = self
            .registry
            .find_by_source_path(&rel)
            .unwrap_or_else(|| {
                panic!("mesh not in registry: {} (rel: {})", path.display(), rel.display())
            });

        let guid = record.guid;

        if let Some(&handle) = self.guid_to_mesh.get(&guid) {
            return handle;
        }

        let cooked = self.project.cooked_path(&guid, "emesh");
        let handle = self
            .asset_manager
            .borrow_mut()
            .load_emesh(&cooked)
            .unwrap_or_else(|| {
                panic!(
                    "cooked mesh missing for '{}' — was cook_pending called at startup? (expected: {})",
                    rel.display(),
                    cooked.display()
                )
            });

        self.guid_to_mesh.insert(guid, handle);
        self.mesh_to_guid.insert(handle, guid);
        handle
    }

    /// Loads a `.emat` file and registers the material. The handle is stable
    /// across sessions via its GUID.
    pub fn load_material<P: AsRef<Path>>(&mut self, path: P) -> MaterialHandle {
        let path = path.as_ref();
        let rel = self.to_rel(path);
        let guid = self.registry.find_by_source_path(&rel).map(|r| r.guid);

        let mat = {
            let mut assets = self.asset_manager.borrow_mut();
            EmatFile::load(path)
                .and_then(|f| f.build_material(&self.project, &self.registry, &mut assets))
                .unwrap_or_else(|e| panic!("failed to load '{}': {}", path.display(), e))
        };
        let handle = self.material_manager.borrow_mut().add_material_instance(mat);

        if let Some(guid) = guid {
            self.guid_to_material.insert(guid, handle);
            self.material_to_guid.insert(handle, guid);
        }

        handle
    }

    /// Returns the stable GUID for a mesh handle loaded via `load_mesh`.
    pub fn mesh_guid(&self, handle: MeshHandle) -> Option<Guid> {
        self.mesh_to_guid.get(&handle).copied()
    }

    /// Returns the stable GUID for a material handle loaded via `load_material`.
    pub fn material_guid(&self, handle: MaterialHandle) -> Option<Guid> {
        self.material_to_guid.get(&handle).copied()
    }

    // ── Internal accessors used by EngineContext ───────────────────────────

    pub(crate) fn raw_assets(&self) -> RefMut<'_, AssetManager> {
        self.asset_manager.borrow_mut()
    }

    pub(crate) fn raw_materials(&self) -> RefMut<'_, MaterialManager> {
        self.material_manager.borrow_mut()
    }

    pub(crate) fn asset_manager_rc(&self) -> Rc<RefCell<AssetManager>> {
        self.asset_manager.clone()
    }

    pub(crate) fn material_manager_rc(&self) -> Rc<RefCell<MaterialManager>> {
        self.material_manager.clone()
    }

    // ── Path helpers ───────────────────────────────────────────────────────

    /// Converts any path (absolute or relative to project root) to a path
    /// relative to the content directory, which is what the registry uses.
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