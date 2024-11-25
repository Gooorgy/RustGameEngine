use core::slice;

use ash::vk::{self};

use super::{buffer::BufferInfo, structs::UniformBufferObject};

pub struct CustomDescriptorPool<'a> {
    logical_device: &'a ash::Device,
    pub descriptor_pool: vk::DescriptorPool,
}

impl<'a> CustomDescriptorPool<'a> {
    pub fn new(logical_device: &'a ash::Device, descriptor_count: u32) -> Self {
        let descriptor_pool_size =
            vk::DescriptorPoolSize::default().descriptor_count(descriptor_count).ty(vk::DescriptorType::UNIFORM_BUFFER);
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(slice::from_ref(&descriptor_pool_size))
            .max_sets(descriptor_count);

        let descriptor_pool = unsafe {
            logical_device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("failed to create descriptor pool!")
        };

        Self {
            logical_device,
            descriptor_pool,
        }
    }

    pub fn allocate_descriptor(
        &self,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &[BufferInfo],
        descriptor_count: u32,
    ) -> Vec<vk::DescriptorSet> {
        let mut layouts = vec![];
        for _ in 0..descriptor_count {
            layouts.push(descriptor_set_layout);
        }

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe {
            self.logical_device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to create descriptor sets")
        };

        for i in 0..descriptor_count {
            let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(uniform_buffers[i as usize].buffer)
                .offset(0)
                .range(std::mem::size_of::<UniformBufferObject>() as u64);

            let write_descriptor_set = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i as usize])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(slice::from_ref(&buffer_info));

            unsafe {
                self.logical_device
                    .update_descriptor_sets(slice::from_ref(&write_descriptor_set), &[]);
            }
        }

        descriptor_sets
    }
}
