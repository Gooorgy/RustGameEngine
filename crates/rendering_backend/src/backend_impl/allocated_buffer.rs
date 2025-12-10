use super::utils;
use crate::backend_impl::device::DeviceInfo;
use crate::buffer::{BufferDesc, BufferUsageFlags};
use crate::memory::MemoryHint;
use ash::{vk, Instance};
use core::panic;
use std::ffi::c_void;
use std::slice;

pub struct AllocatedBuffer {
    pub buffer: vk::Buffer,
    pub buffer_memory: vk::DeviceMemory,
    pub buffer_size: vk::DeviceSize,
    pub mapped_buffer: Option<*mut c_void>,
}

impl AllocatedBuffer {
    pub fn new<T>(
        device_info: &DeviceInfo,
        instance: &Instance,
        buffer_desc: BufferDesc,
        initial_data: Option<&[T]>,
    ) -> Self {
        let buffer_size = buffer_desc.size as vk::DeviceSize;

        let (buffer, memory) = if let Some(data) = initial_data {
            if matches!(buffer_desc.memory_hint, MemoryHint::GPUOnly) {
                Self::create_device_local_buffer_with_staging(
                    device_info,
                    instance,
                    &buffer_desc,
                    data,
                )
            } else {
                Self::create_host_visible_buffer(device_info, instance, &buffer_desc, initial_data)
            }
        } else {
            Self::create_empty_buffer(device_info, instance, &buffer_desc)
        };

        let mapped_buffer = if matches!(buffer_desc.memory_hint, MemoryHint::CPUWritable) {
            let x = unsafe {
                device_info
                    .logical_device
                    .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                    .expect("failed to map memory")
            };
            
            Some(x)
        } else {
            None
        };

        AllocatedBuffer {
            buffer,
            buffer_memory: memory,
            mapped_buffer,
            buffer_size,
        }
    }

    pub fn update_buffer<T>(&mut self, data: &[T]) {
        if let Some(mapped) = self.mapped_buffer {
            unsafe {
                (mapped as *mut T).copy_from_nonoverlapping(data.as_ptr(), data.len());
            }
        }
    }

    pub fn flush_mapped_memory_ranges(
        &mut self,
        device: &ash::Device,
        mapped_memory_range: vk::MappedMemoryRange,
    ) {
        unsafe {
            device
                .flush_mapped_memory_ranges(&[mapped_memory_range])
                .expect("d");
        }
    }

    pub fn create_buffer(
        instance: &Instance,
        device_info: &DeviceInfo,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device_info
                .logical_device
                .create_buffer(&buffer_create_info, None)
                .expect("failed to create buffer")
        };

        let mem_requirements = unsafe {
            device_info
                .logical_device
                .get_buffer_memory_requirements(buffer)
        };
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device_info._physical_device) };
        let memory_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(utils::find_memory_type(
                mem_requirements.memory_type_bits,
                memory_property_flags,
                memory_properties,
            ));

        let buffer_memory = unsafe {
            device_info
                .logical_device
                .allocate_memory(&memory_alloc_info, None)
                .expect("failed to allocate memory")
        };

        unsafe {
            device_info
                .logical_device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("failed to bind buffer");
        }

        (buffer, buffer_memory)
    }

    fn create_empty_buffer(
        device_info: &DeviceInfo,
        instance: &Instance,
        desc: &BufferDesc,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_info = vk::BufferCreateInfo {
            size: desc.size as u64,
            usage: map_usage_flags(desc.usage),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe {
            device_info
                .logical_device
                .create_buffer(&buffer_info, None)
                .unwrap()
        };

        let mem_requirements = unsafe {
            device_info
                .logical_device
                .get_buffer_memory_requirements(buffer)
        };
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device_info._physical_device) };

        let memory_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(utils::find_memory_type(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                memory_properties,
            ));

        let memory = unsafe {
            device_info
                .logical_device
                .allocate_memory(&memory_alloc_info, None)
                .unwrap()
        };
        unsafe {
            device_info
                .logical_device
                .bind_buffer_memory(buffer, memory, 0)
                .unwrap()
        };

        (buffer, memory)
    }

    fn create_host_visible_buffer<T>(
        device_info: &DeviceInfo,
        instance: &Instance,
        desc: &BufferDesc,
        initial_data: Option<&[T]>,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        // 1. Create VkBuffer
        let buffer_info = vk::BufferCreateInfo {
            size: desc.size as u64,
            usage: map_usage_flags(desc.usage), // usage flags mapped
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe {
            device_info
                .logical_device
                .create_buffer(&buffer_info, None)
                .unwrap()
        };

        // 2. Allocate host-visible memory
        let mem_requirements = unsafe {
            device_info
                .logical_device
                .get_buffer_memory_requirements(buffer)
        };

        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device_info._physical_device) };

        let memory_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(utils::find_memory_type(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                memory_properties,
            ));

        let memory = unsafe {
            device_info
                .logical_device
                .allocate_memory(&memory_alloc_info, None)
                .unwrap()
        };
        unsafe {
            device_info
                .logical_device
                .bind_buffer_memory(buffer, memory, 0)
                .unwrap()
        };

        // 3. Optionally copy initial data
        if let Some(data) = initial_data {
            let ptr = unsafe {
                device_info
                    .logical_device
                    .map_memory(memory, 0, desc.size as u64, vk::MemoryMapFlags::empty())
                    .unwrap() as *mut T
            };
            unsafe {
                ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
                device_info.logical_device.unmap_memory(memory);
            }
        }

        (buffer, memory)
    }

    fn create_device_local_buffer_with_staging<T>(
        device_info: &DeviceInfo,
        instance: &Instance,
        desc: &BufferDesc,
        data: &[T],
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let staging_desc = BufferDesc {
            size: desc.size,
            usage: BufferUsageFlags::TRANSFER_SRC,
            memory_hint: MemoryHint::CPUToGPU,
        };

        let (staging_buffer, staging_memory) =
            Self::create_host_visible_buffer(device_info, instance, &staging_desc, Some(data));

        let device_desc = BufferDesc {
            size: desc.size,
            usage: desc.usage | BufferUsageFlags::TRANSFER_DST,
            memory_hint: MemoryHint::GPUOnly,
        };

        let (device_buffer, device_memory) =
            Self::create_empty_buffer(device_info, instance, &device_desc);

        Self::copy_buffer(device_info, staging_buffer, device_buffer, desc.size);

        Self::destroy_buffer(staging_buffer, staging_memory, &device_info.logical_device);

        (device_buffer, device_memory)
    }

    pub fn copy_buffer(device_info: &DeviceInfo, src: vk::Buffer, dst: vk::Buffer, size: usize) {
        let command_buffer = Self::begin_single_time_command(&device_info);

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: size as u64,
        };

        unsafe {
            device_info
                .logical_device
                .cmd_copy_buffer(command_buffer, src, dst, &[copy_region]);
        }

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

    pub fn end_single_time_command(device_info: &DeviceInfo, command_buffer: vk::CommandBuffer) {
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

    pub fn destroy_buffer(
        buffer: vk::Buffer,
        buffer_memory: vk::DeviceMemory,
        logical_device: &ash::Device,
    ) {
        unsafe {
            logical_device.destroy_buffer(buffer, None);
            logical_device.free_memory(buffer_memory, None);
        };
    }
}

pub struct BufferInfo {
    pub buffer: vk::Buffer,
    pub buffer_memory: vk::DeviceMemory,
}

impl BufferInfo {
    pub fn new(
        instance: &Instance,
        device_info: &DeviceInfo,
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

fn map_usage_flags(usage: BufferUsageFlags) -> vk::BufferUsageFlags {
    let mut flags = vk::BufferUsageFlags::empty();
    if usage.contains(BufferUsageFlags::VERTEX_BUFFER) {
        flags |= vk::BufferUsageFlags::VERTEX_BUFFER;
    }
    if usage.contains(BufferUsageFlags::INDEX_BUFFER) {
        flags |= vk::BufferUsageFlags::INDEX_BUFFER;
    }
    if usage.contains(BufferUsageFlags::UNIFORM) {
        flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
    }
    if usage.contains(BufferUsageFlags::STORAGE) {
        flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
    }
    if usage.contains(BufferUsageFlags::TRANSFER_SRC) {
        flags |= vk::BufferUsageFlags::TRANSFER_SRC;
    }
    if usage.contains(BufferUsageFlags::TRANSFER_DST) {
        flags |= vk::BufferUsageFlags::TRANSFER_DST;
    }
    flags
}
