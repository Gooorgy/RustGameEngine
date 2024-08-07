use ash::vk;
use core::panic;

use crate::VERTICES;

use super::{device, structs::Vertex};

pub struct VertexBufferInfo {
    pub buffer: vk::Buffer,
    _buffer_memory: vk::DeviceMemory,
}

impl VertexBufferInfo {
    pub fn new(instance: &ash::Instance, device_info: &device::DeviceInfo) -> Self {
        let buffer_create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: std::mem::size_of_val(&VERTICES) as u64,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

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

        let required_memory_flags: vk::MemoryPropertyFlags =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        let memory_type_index = Self::find_memory_type(
            memory_requirements.memory_type_bits,
            required_memory_flags,
            memory_properties,
        );

        let memory_allocate_create_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index,
            ..Default::default()
        };

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

        unsafe {
            let data = device_info
                .logical_device
                .map_memory(
                    buffer_memory,
                    0,
                    buffer_create_info.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to map memory") as *mut Vertex;

            data.copy_from_nonoverlapping(VERTICES.as_ptr(), VERTICES.len());

            device_info.logical_device.unmap_memory(buffer_memory);
        };

        Self {
            buffer,
            _buffer_memory: buffer_memory,
        }
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
