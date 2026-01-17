#[derive(Debug)]
pub struct Entity(pub usize);

impl Entity {
    pub fn index(&self) -> usize {
        self.0
    }
}
