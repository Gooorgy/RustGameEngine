use std::ptr;

use crate::{render_pass, structs::QueueFamiliyIndices};

pub fn create_command_pool(
    device: &ash::Device,
    queue_family_indeces: &QueueFamiliyIndices,
) -> ash::vk::CommandPool {
    let command_pool_create_info = ash::vk::CommandPoolCreateInfo {
        s_type: ash::vk::StructureType::COMMAND_POOL_CREATE_INFO,
        flags: ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index: queue_family_indeces.graphics_family.expect(""),
        ..Default::default()
    };

    unsafe {
        device
            .create_command_pool(&command_pool_create_info, None)
            .expect("Unable to create command pool")
    }
}

pub fn create_command_buffer(
    device: &ash::Device,
    command_pool: &ash::vk::CommandPool,
) -> Vec<ash::vk::CommandBuffer> {
    let command_buffer_allocate_info = ash::vk::CommandBufferAllocateInfo {
        s_type: ash::vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        command_pool: *command_pool,
        level: ash::vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: 1,
        ..Default::default()
    };

    unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Unable to allocate command buffers")
    }
}

pub fn record_command_buffer(device: &ash::Device, command_buffers: &Vec<ash::vk::CommandBuffer>) {
    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo {
            s_type: ash::vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags: ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
            p_inheritance_info: ptr::null(),
            p_next: ptr::null(),
            ..Default::default()
        };

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Unable to begin recording command buffer")
        }
    }
}
