use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct AssetId<T> {
    pub id: u64,
    _marker: PhantomData<T>,
}

impl<T> AssetId<T> {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn raw(&self) -> u64 {
        self.id
    }
}

impl<T> Clone for AssetId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AssetId<T> {}

impl<T> PartialEq<Self> for AssetId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for AssetId<T> {}

impl<T> Hash for AssetId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
