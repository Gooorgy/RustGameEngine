pub mod emat;
pub mod mesh_conditioner;
mod shader;
pub mod texture_conditioner;

pub use emat::{EmatError, EmatFile};
pub use mesh_conditioner::{MeshConditionError, MeshConditioner};
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
            _ => {}
        }
    }
}
