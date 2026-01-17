#[derive(Copy, Clone, Debug)]
pub struct GpuImageHandle(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct ImageDesc {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub is_cubemap: bool,
    pub format: TextureFormat,
    pub aspect: ImageAspect,
    pub usage: ImageUsageFlags,
    pub clear_value: Option<ClearValue>,
}

#[derive(Clone, Copy, Debug)]
pub enum TextureFormat {
    R8g8b8a8Unorm,
    R16g16b16a16Float,
    D32Float,
    // add others as needed
}

bitflags::bitflags! {
    #[derive(Clone,Copy, Debug)]
    pub struct ImageUsageFlags: u32 {
        const COLOR_ATTACHMENT   = 0b0001;
        const DEPTH_ATTACHMENT   = 0b0010;
        const SAMPLED            = 0b0100;
        const STORAGE            = 0b1000;
        const TRANSFER_SRC       = 0b0001_0000;
        const TRANSFER_DST       = 0b0010_0000;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SampleCount {
    Sample1,
    Sample2,
    Sample4,
    Sample8,
    Sample16,
}

#[derive(Clone, Copy, Debug)]
pub enum ClearValue {
    Color([f32; 4]),
    DepthStencil { depth: f32, stencil: u32 },
}

#[derive(Clone, Copy, Debug)]
pub enum ImageAspect {
    Color,
    Depth,
    Stencil,
    DepthStencil,
}
