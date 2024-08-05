use std::ptr;

use ash::vk;

use super::{constants, device::DeviceInfo};

pub struct CommandBufferInfo {
    pub command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandBufferInfo {
    pub fn new(device_info: &DeviceInfo) -> CommandBufferInfo {
        let command_buffer_allocate_info = ash::vk::CommandBufferAllocateInfo {
            s_type: ash::vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: device_info.command_pool,
            level: ash::vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: constants::MAX_FRAMES_IN_FLIGHT,
            ..Default::default()
        };

        let command_buffers = unsafe {
            device_info
                .logical_device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Unable to allocate command buffers")
        };

        Self { command_buffers }
    }

    pub fn record_command_buffer(&self, device: &ash::Device, current_frame: u32) {
        let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo {
            s_type: ash::vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags: ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
            p_inheritance_info: ptr::null(),
            p_next: ptr::null(),
            ..Default::default()
        };

        unsafe {
            device
                .begin_command_buffer(
                    self.command_buffers[current_frame as usize],
                    &command_buffer_begin_info,
                )
                .expect("Unable to begin recording command buffer")
        }
    }
}
