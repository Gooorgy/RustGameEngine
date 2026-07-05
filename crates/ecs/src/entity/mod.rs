#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct Entity(pub usize);

impl Entity {
    pub fn index(&self) -> usize {
        self.0
    }
}
