use crate::memory::MemoryHint;

#[derive(Copy, Clone, Debug)]
pub struct BufferHandle(pub usize);

pub struct BufferDesc {
    pub size: usize,
    pub usage: BufferUsageFlags,
    pub memory_hint: MemoryHint,
}
bitflags::bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct BufferUsageFlags: u32 {
        const VERTEX_BUFFER = 0b0001;
        const INDEX_BUFFER  = 0b0010;
        const UNIFORM       = 0b0100;
        const STORAGE       = 0b1000;
        const TRANSFER_SRC  = 0b0001_0000;
        const TRANSFER_DST  = 0b0010_0000;
    }
}
