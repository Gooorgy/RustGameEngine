use crate::assets::asset_manager::AssetManager;

pub struct System {
    asset_manager: AssetManager,
}

impl HasAssetManager for System {
    fn asset_manager(&self) -> &AssetManager {
        &self.asset_manager
    }
}

impl HasGameState for System {
    fn game_state(&self) -> bool {
        true
    }
}

pub trait HasAssetManager {
    fn asset_manager(&self) -> &AssetManager;
}

pub trait HasGameState {
    fn game_state(&self) -> bool;
}

pub trait InitReq: HasAssetManager {}
