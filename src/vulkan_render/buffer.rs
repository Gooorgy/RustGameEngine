use super::device;
use crate::vulkan_render::device::DeviceInfo;
use ash::vk;
use core::panic;
use std::slice;

pub struct BufferInfo {
    pub buffer: vk::Buffer,
    pub buffer_memory: vk::DeviceMemory,
}

impl BufferInfo {
    pub fn new(
        instance: &ash::Instance,
        device_info: &device::DeviceInfo,
        size: u64,
        memory_properties_flags: vk::MemoryPropertyFlags,
        usage: vk::BufferUsageFlags,
    ) -> Self {
        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device_info
                .logical_device
                .create_buffer(&buffer_create_info, None)
                .expect("Failed to create vertex buffer")
        };

        let memory_requirements = unsafe {
            device_info
                .logical_device
                .get_buffer_memory_requirements(buffer)
        };
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device_info._physical_device) };

        let memory_type_index = Self::find_memory_type(
            memory_requirements.memory_type_bits,
            memory_properties_flags,
            memory_properties,
        );

        let memory_allocate_create_info = vk::MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);

        let buffer_memory = unsafe {
            device_info
                .logical_device
                .allocate_memory(&memory_allocate_create_info, None)
                .expect("Failed to allocate vertex buffer memory")
        };

        unsafe {
            device_info
                .logical_device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bin buffer memory");
        }

        Self {
            buffer,
            buffer_memory,
        }
    }

    pub fn copy_buffer(
        &self,
        src_buffer: &BufferInfo,
        size: u64,
        device_info: &device::DeviceInfo,
    ) {
        let command_buffer = Self::begin_single_time_command(&device_info);

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        };

        unsafe {
            device_info.logical_device.cmd_copy_buffer(
                command_buffer,
                src_buffer.buffer,
                self.buffer,
                &[copy_region],
            )
        };

        Self::end_single_time_command(device_info, command_buffer);
    }

    pub fn begin_single_time_command(device_info: &DeviceInfo) -> vk::CommandBuffer {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1)
            .command_pool(device_info.command_pool);

        let command_buffer = unsafe {
            device_info
                .logical_device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffer!")
        };

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device_info
                .logical_device
                .begin_command_buffer(command_buffer[0], &begin_info)
                .expect("Failed to begin command buffer recording!");
        }

        command_buffer[0]
    }

    pub fn end_single_time_command(
        device_info: &DeviceInfo,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            device_info
                .logical_device
                .end_command_buffer(command_buffer)
                .expect("Failed to end command buffer!");
        };

        let submit_info =
            vk::SubmitInfo::default().command_buffers(slice::from_ref(&command_buffer));
        unsafe {
            device_info
                .logical_device
                .queue_submit(
                    device_info.queue_info.graphics_queue,
                    &[submit_info],
                    vk::Fence::null(),
                )
                .expect("Failed to submit queue!");
            device_info
                .logical_device
                .queue_wait_idle(device_info.queue_info.graphics_queue)
                .expect("Failed to wait on queue!");
            device_info
                .logical_device
                .free_command_buffers(device_info.command_pool, &[command_buffer]);
        }
    }

    pub fn destroy_buffer(self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_buffer(self.buffer, None);
            logical_device.free_memory(self.buffer_memory, None);
        };
    }

    fn find_memory_type(
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> u32 {
        for (i, memory_type) in memory_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(properties) {
                return i as u32;
            }
        }

        panic!()
    }
}
