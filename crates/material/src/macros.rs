#[macro_export]
macro_rules! assign_texture {
    ($texture_name:expr, $asset_manager:expr, $texture_asset:expr) => {
        if let Some(texture_name) = $texture_name {
            match $asset_manager.get_image(texture_name) {
                Some(asset) => {
                    $texture_asset = Some(asset);
                }
                None => {
                    eprintln!(
                        "Warning: Could not load texture '{}'.",
                        stringify!($texture_name)
                    );
                }
            }
        }
    };
}
