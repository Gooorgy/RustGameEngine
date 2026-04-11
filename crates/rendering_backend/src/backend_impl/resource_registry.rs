use crate::backend_impl::allocated_buffer::AllocatedBuffer;
use crate::backend_impl::descriptor_info::{
    AllocatedDescriptorSet, DescriptorLayoutInfo, DescriptorPoolChunk,
};
use crate::backend_impl::destroyable::Destroyable;
use crate::backend_impl::image_util::AllocatedImage;
use crate::backend_impl::pipeline_info::PipelineInfo;
use crate::buffer::BufferHandle;
use crate::descriptor::{DescriptorLayoutHandle, DescriptorSetHandle};
use crate::image::GpuImageHandle;
use crate::pipeline::PipelineHandle;
use crate::sampler::SamplerHandle;
use ash::vk;

/// Thin wrapper so `vk::Sampler` (a plain handle) can implement `Destroyable`.
struct OwnedSampler(vk::Sampler);

impl Destroyable for OwnedSampler {
    fn destroy(&self, device: &ash::Device) {
        unsafe { device.destroy_sampler(self.0, None); }
    }
}

pub struct ResourceRegistry {
    pub images: Vec<AllocatedImage>,
    pub buffers: Vec<AllocatedBuffer>,
    pub descriptor_pools: Vec<DescriptorPoolChunk>,
    pub descriptor_sets: Vec<AllocatedDescriptorSet>,
    pub descriptor_layouts: Vec<DescriptorLayoutInfo>,
    pub pipelines: Vec<PipelineInfo>,
    pub samplers: Vec<vk::Sampler>,
    /// Resources waiting to be freed after the next GPU fence wait.
    pending_destroy: Vec<Box<dyn Destroyable>>,
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
            pending_destroy: vec![],
        }
    }

    pub fn register_image(&mut self, image: AllocatedImage) -> GpuImageHandle {
        let id = self.images.len();
        self.images.push(image);
        GpuImageHandle(id)
    }

    pub fn register_buffer(&mut self, buffer: AllocatedBuffer) -> BufferHandle {
        let id = self.buffers.len();
        self.buffers.push(buffer);
        BufferHandle(id)
    }

    pub fn register_allocated_descriptor_set(
        &mut self,
        allocated_descriptor: AllocatedDescriptorSet,
    ) -> DescriptorSetHandle {
        let id = self.descriptor_sets.len();
        self.descriptor_sets.push(allocated_descriptor);
        DescriptorSetHandle(id)
    }

    pub fn register_descriptor_layout(
        &mut self,
        layout_info: DescriptorLayoutInfo,
    ) -> DescriptorLayoutHandle {
        let id = self.descriptor_layouts.len();
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

    /// Queue any resource for deferred destruction. The resource will be freed
    /// on the next call to `flush_pending`, which happens after the GPU fence wait.
    #[allow(dead_code)]
    pub fn queue_destroy(&mut self, resource: Box<dyn Destroyable>) {
        self.pending_destroy.push(resource);
    }

    /// Free all queued resources. Call this immediately after the per-frame fence wait
    /// to guarantee the GPU has finished using these resources.
    pub fn flush_pending(&mut self, device: &ash::Device) {
        for resource in self.pending_destroy.drain(..) {
            resource.destroy(device);
        }
    }

    /// Free every live resource and flush the pending queue.
    /// Call this on shutdown after `device_wait_idle`.
    pub fn destroy_all(&mut self, device: &ash::Device) {
        self.flush_pending(device);
        // Free individual sets before destroying their pools.
        for set in self.descriptor_sets.drain(..) {
            set.destroy(device);
        }
        for pool in self.descriptor_pools.drain(..) {
            pool.destroy(device);
        }
        for layout in self.descriptor_layouts.drain(..) {
            layout.destroy(device);
        }
        for pipeline in self.pipelines.drain(..) {
            pipeline.destroy(device);
        }
        for image in self.images.drain(..) {
            image.destroy(device);
        }
        for buffer in self.buffers.drain(..) {
            buffer.destroy(device);
        }
        for sampler in self.samplers.drain(..) {
            OwnedSampler(sampler).destroy(device);
        }
    }
}
