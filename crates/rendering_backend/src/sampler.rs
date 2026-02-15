#[derive(Copy, Clone, Debug)]
pub struct SamplerHandle(pub usize);

#[derive(Clone, Copy, Debug)]
pub enum Filter {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug)]
pub enum SamplerAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

#[derive(Clone, Copy, Debug)]
pub struct SamplerDesc {
    pub mag_filter: Filter,
    pub min_filter: Filter,
    pub address_u: SamplerAddressMode,
    pub address_v: SamplerAddressMode,
    pub address_w: SamplerAddressMode,
    pub compare_enable: bool,
    pub compare_op: Option<CompareOp>,
}

use crate::pipeline::CompareOp;
