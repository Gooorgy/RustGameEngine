use ash::vk;
use core::panic;

use super::device;

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
        let buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(device_info.command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffer = unsafe {
            device_info
                .logical_device
                .allocate_command_buffers(&buffer_allocate_info)
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

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        };

        unsafe {
            device_info.logical_device.cmd_copy_buffer(
                command_buffer[0],
                src_buffer.buffer,
                self.buffer,
                &[copy_region],
            )
        };

        unsafe {
            device_info
                .logical_device
                .end_command_buffer(command_buffer[0])
                .expect("Failed to end command buffer recording!");
        }

        let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffer);

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
                .expect("Failed to idle?!");

            device_info
                .logical_device
                .free_command_buffers(device_info.command_pool, &command_buffer);
        };
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
