use crate::AssetManager;

pub struct Resolved<T: Resolve> {
    pub data: T,
    pub handles: T::Handles,
}

pub trait Resolve {
    type Handles;

    fn resolve(&self, asset_manager: &mut AssetManager) -> Self::Handles;
}
