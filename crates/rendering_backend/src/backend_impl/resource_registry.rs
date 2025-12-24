use crate::backend_impl::allocated_buffer::AllocatedBuffer;
use crate::backend_impl::descriptor_info::{
    AllocatedDescriptorSet, DescriptorLayoutInfo, DescriptorPoolChunk,
};
use crate::backend_impl::image_util::AllocatedImage;
use crate::backend_impl::pipeline_info::PipelineInfo;
use crate::buffer::BufferHandle;
use crate::descriptor::{DescriptorLayoutHandle, DescriptorSetHandle};
use crate::image::GpuImageHandle;
use crate::pipeline::PipelineHandle;
use crate::sampler::SamplerHandle;
use ash::vk;

pub struct ResourceRegistry {
    pub images: Vec<AllocatedImage>,
    pub buffers: Vec<AllocatedBuffer>,
    pub descriptor_pools: Vec<DescriptorPoolChunk>,
    pub descriptor_sets: Vec<AllocatedDescriptorSet>,
    pub descriptor_layouts: Vec<DescriptorLayoutInfo>,
    pub pipelines: Vec<PipelineInfo>,
    pub samplers: Vec<vk::Sampler>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            images: vec![],
            buffers: vec![],
            descriptor_pools: vec![],
            descriptor_sets: vec![],
            descriptor_layouts: vec![],
            pipelines: vec![],
            samplers: vec![],
        }
    }

    pub fn register_image(&mut self, image: AllocatedImage) -> GpuImageHandle {
        let id = self.images.len(); // unique ID
        self.images.push(image);
        GpuImageHandle(id)
    }

    pub fn register_buffer(&mut self, buffer: AllocatedBuffer) -> BufferHandle {
        let id = self.buffers.len(); // unique ID
        self.buffers.push(buffer);
        BufferHandle(id)
    }

    pub fn register_allocated_descriptor_set(
        &mut self,
        allocated_descriptor: AllocatedDescriptorSet,
    ) -> DescriptorSetHandle {
        let id = self.descriptor_sets.len(); // unique ID
        self.descriptor_sets.push(allocated_descriptor);
        DescriptorSetHandle(id)
    }

    pub fn register_descriptor_layout(
        &mut self,
        layout_info: DescriptorLayoutInfo,
    ) -> DescriptorLayoutHandle {
        let id = self.descriptor_layouts.len(); // unique ID
        self.descriptor_layouts.push(layout_info);
        DescriptorLayoutHandle(id)
    }

    pub fn register_sampler(&mut self, sampler: vk::Sampler) -> SamplerHandle {
        let id = self.samplers.len();
        self.samplers.push(sampler);
        SamplerHandle(id)
    }

    pub fn register_pipeline(&mut self, pipeline: PipelineInfo) -> PipelineHandle {
        let id = self.pipelines.len();
        self.pipelines.push(pipeline);
        PipelineHandle(id)
    }
}
