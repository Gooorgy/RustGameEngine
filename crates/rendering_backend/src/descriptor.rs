use crate::buffer::BufferHandle;
use crate::image::GpuImageHandle;
use crate::sampler::SamplerHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorSetHandle(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorLayoutHandle(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorType {
    UniformBuffer,
    StorageBuffer,
    SampledImage,
    Sampler,
    CombinedImageSampler,
    StorageImage,
}

pub enum DescriptorWrite {
    UniformBuffer(u32, BufferHandle),
    // SampledImage(u32, ImageHandle, SamplerHandle),
    CombinedImageSampler(u32, GpuImageHandle, SamplerHandle),
}

bitflags::bitflags! {
    #[derive(Clone,Copy, Debug)]
    pub struct ShaderStage: u32 {
        const VERTEX   = 0b0001;
        const FRAGMENT = 0b0010;
        const COMPUTE  = 0b0100;
    }
}

#[derive(Clone, Debug)]
pub struct DescriptorBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub count: u32,
    pub stages: ShaderStage,
}

#[derive(Clone, Debug)]
pub struct DescriptorLayoutDesc {
    pub bindings: Vec<DescriptorBinding>,
}

pub enum DescriptorValue {
    UniformBuffer(BufferHandle),
    StorageBuffer(BufferHandle),
    SampledImage(SampledImageInfo),
}

#[derive(Copy, Clone, Debug)]
pub struct SampledImageInfo {
    pub image: GpuImageHandle,
    pub sampler: SamplerHandle,
}

pub struct DescriptorWriteDesc {
    pub binding: usize,
    pub value: DescriptorValue,
}
