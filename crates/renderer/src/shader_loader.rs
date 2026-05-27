use material::ShaderRef;
use std::collections::HashMap;
use std::path::PathBuf;

/// Reads SPV bytecode on demand for user asset shaders, caching each file after first load.
/// Engine built-in shaders are served from bytes embedded at compile time.
pub(crate) struct ShaderCache {
    asset_bytes: HashMap<PathBuf, Vec<u8>>,
    cache_dir: PathBuf,
}

impl ShaderCache {
    pub(crate) fn new(cache_dir: PathBuf) -> Self {
        Self {
            asset_bytes: HashMap::new(),
            cache_dir,
        }
    }

    /// Returns SPV bytes for `shader_ref`.
    /// Built-in shaders are served from bytes embedded in the binary.
    /// Asset shaders are read from the project cache directory on the first call, then cached.
    /// `active_defines` selects the compiled permutation for both shader types.
    /// Panics if a file cannot be read or a built-in name is unknown.
    pub(crate) fn load(&mut self, shader_ref: &ShaderRef, active_defines: &[String]) -> Vec<u8> {
        match shader_ref {
            ShaderRef::BuiltIn(name) => {
                let key = resolve_name(name, active_defines);
                builtin_bytes(&key).to_vec()
            }
            ShaderRef::Asset(guid) => {
                let path = resolve_asset_path(&self.cache_dir, &guid.to_string(), active_defines);
                self.asset_bytes
                    .entry(path.clone())
                    .or_insert_with(|| {
                        let msg = format!("Failed to load asset shader '{}'", path.display());
                        std::fs::read(&path).expect(&msg)
                    })
                    .clone()
            }
        }
    }
}

fn resolve_asset_path(cache_dir: &PathBuf, base: &str, active_defines: &[String]) -> PathBuf {
    let mut sorted = active_defines.to_vec();
    sorted.sort_unstable();
    if sorted.is_empty() {
        cache_dir.join(format!("{base}.spv"))
    } else {
        cache_dir.join(format!("{base}.{}.spv", sorted.join(".")))
    }
}

fn resolve_name(base: &str, active_defines: &[String]) -> String {
    let mut sorted = active_defines.to_vec();
    sorted.sort_unstable();
    if sorted.is_empty() {
        base.to_string()
    } else {
        format!("{base}.{}", sorted.join("."))
    }
}

fn builtin_bytes(name: &str) -> &'static [u8] {
    match name {
        "vert"             => include_bytes!("../shaders/vert.spv"),
        "shadow"           => include_bytes!("../shaders/shadow.spv"),
        "quad"             => include_bytes!("../shaders/quad.spv"),
        "lighting"         => include_bytes!("../shaders/lighting.spv"),
        "line_debug_vert"  => include_bytes!("../shaders/line_debug_vert.spv"),
        "line_debug_frag"  => include_bytes!("../shaders/line_debug_frag.spv"),
        "pbr.frag"         => include_bytes!("../shaders/pbr.frag.spv"),
        "pbr.frag.HAS_COLOR_TEXTURE"
            => include_bytes!("../shaders/pbr.frag.HAS_COLOR_TEXTURE.spv"),
        "pbr.frag.HAS_NORMAL_TEXTURE"
            => include_bytes!("../shaders/pbr.frag.HAS_NORMAL_TEXTURE.spv"),
        "pbr.frag.HAS_ORM_TEXTURE"
            => include_bytes!("../shaders/pbr.frag.HAS_ORM_TEXTURE.spv"),
        "pbr.frag.HAS_COLOR_TEXTURE.HAS_NORMAL_TEXTURE"
            => include_bytes!("../shaders/pbr.frag.HAS_COLOR_TEXTURE.HAS_NORMAL_TEXTURE.spv"),
        "pbr.frag.HAS_COLOR_TEXTURE.HAS_ORM_TEXTURE"
            => include_bytes!("../shaders/pbr.frag.HAS_COLOR_TEXTURE.HAS_ORM_TEXTURE.spv"),
        "pbr.frag.HAS_NORMAL_TEXTURE.HAS_ORM_TEXTURE"
            => include_bytes!("../shaders/pbr.frag.HAS_NORMAL_TEXTURE.HAS_ORM_TEXTURE.spv"),
        "pbr.frag.HAS_COLOR_TEXTURE.HAS_NORMAL_TEXTURE.HAS_ORM_TEXTURE"
            => include_bytes!("../shaders/pbr.frag.HAS_COLOR_TEXTURE.HAS_NORMAL_TEXTURE.HAS_ORM_TEXTURE.spv"),
        other => panic!("unknown built-in shader: '{other}'"),
    }
}
