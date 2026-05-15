use crate::handle::Handle;

#[derive(Clone, Debug)]
pub struct ImageData {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub type ImageHandle = Handle<ImageData>;