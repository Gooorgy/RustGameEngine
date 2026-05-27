pub mod emat;
pub mod mesh_conditioner;
mod shader;
pub mod shader_conditioner;
pub mod texture_conditioner;

pub use emat::{EmatError, EmatFile};
pub use mesh_conditioner::{MeshConditionError, MeshConditioner};
pub use shader_conditioner::{ShaderConditionError, ShaderConditioner};
pub use texture_conditioner::{TextureConditionError, TextureConditioner};

use project::{AssetRegistry, AssetType, Project};

/// Cooks all assets in the registry that are `Uncooked` or `Dirty`.
/// Errors for individual assets are logged but do not abort the batch.
pub fn cook_pending(registry: &AssetRegistry, project: &Project) {
    for record in registry.pending() {
        match record.asset_type {
            AssetType::Mesh => {
                let src = project.content_dir.join(&record.source_path);
                let dst = project.cooked_path(&record.guid, "emesh");
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
                let src = project.content_dir.join(&record.source_path);
                let dst = project.cooked_path(&record.guid, "etex");
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
                let src = project.content_dir.join(&record.source_path);
                let shader_cache_dir = project.cache_dir.join("shaders");
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