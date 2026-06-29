pub mod codegen;
pub mod emat;
pub mod mesh_conditioner;
mod shader;
pub mod shader_conditioner;
pub mod texture_conditioner;

pub use emat::{EmatError, EmatFile};
pub use mesh_conditioner::{MeshConditionError, MeshConditioner};
pub use shader_conditioner::{ShaderConditionError, ShaderConditioner};
use std::path::{Path, PathBuf};
use common::Guid;
pub use texture_conditioner::{TextureConditionError, TextureConditioner};

use project::{AssetRegistry, AssetType};

/// Cooks all assets in the registry that are `Uncooked` or `Dirty`.
/// Errors for individual assets are logged but do not abort the batch.
pub fn cook_pending(registry: &AssetRegistry, cache_dir: &Path, content_dir: &Path) {
    for record in registry.pending() {
        match record.asset_type {
            AssetType::Mesh => {
                let src = content_dir.join(&record.source_path);
                let dst = resolve_cooked_path(cache_dir, &record.guid, "emesh");
                match MeshConditioner::condition(&src, &dst) {
                    Ok(()) => println!("cooked mesh: {}", record.source_path.display()),
                    Err(e) => eprintln!(
                        "warning: failed to cook '{}': {}",
                        record.source_path.display(),
                        e
                    ),
                }
            }
            AssetType::Texture => {
                let src = content_dir.join(&record.source_path);
                let dst = resolve_cooked_path(cache_dir, &record.guid, "etex");
                match TextureConditioner::condition(&src, &dst) {
                    Ok(()) => println!("cooked texture: {}", record.source_path.display()),
                    Err(e) => eprintln!(
                        "warning: failed to cook '{}': {}",
                        record.source_path.display(),
                        e
                    ),
                }
            }
            AssetType::Shader => {
                let src = content_dir.join(&record.source_path);
                let shader_cache_dir = cache_dir.join("shaders");
                let _ = std::fs::create_dir_all(&shader_cache_dir);
                match ShaderConditioner::condition(&src, record.guid, &shader_cache_dir) {
                    Ok(()) => println!("cooked shader: {}", record.source_path.display()),
                    Err(e) => eprintln!(
                        "warning: failed to cook '{}': {}",
                        record.source_path.display(),
                        e
                    ),
                }
            }
            _ => {}
        }
    }
}

fn resolve_cooked_path(cache_dir: &Path, guid: &Guid, extension: &str) -> PathBuf {
    cache_dir
        .join("cooked")
        .join(format!("{}.{}", guid, extension))
}
