
#[derive(Debug)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    
    pub fn get_aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}