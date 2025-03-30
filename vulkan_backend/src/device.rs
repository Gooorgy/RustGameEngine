use std::{collections::HashSet, ffi::CStr};

use ash::vk;

use super::surface::SurfaceInfo;

const DEVICE_EXTENSIONS: [&CStr; 4] = [
    vk::KHR_SWAPCHAIN_NAME,
    vk::KHR_SYNCHRONIZATION2_NAME,
    vk::KHR_DYNAMIC_RENDERING_NAME,
    vk::EXT_CONSERVATIVE_RASTERIZATION_NAME,
];

pub struct DeviceInfo {
    pub _physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,
    pub queue_info: QueueInfo,
    pub command_pool: vk::CommandPool,
    pub swapchain_support_details: SwapChainSupportDetails,
    pub min_ubo_alignment: u64,
}

impl DeviceInfo {
    pub fn new(instance: &ash::Instance, surface_info: &SurfaceInfo) -> DeviceInfo {
        let physical_device = Self::pick_physical_device(instance, surface_info);
        let swapchain_support_details =
            Self::query_swap_chain_support(physical_device, surface_info);
        // We can safely unwrap because
        let queue_indices =
            Self::find_queue_family(instance, physical_device, surface_info).unwrap();

        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(queue_indices.graphics_queue_index);
        unique_queue_families.insert(queue_indices.present_queue_index);

        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];

        for &unique_queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(unique_queue_family)
                .queue_priorities(&queue_priorities);

            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures::default()
            .sampler_anisotropy(true)
            .depth_clamp(true);

        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);

        let mut vulkan_12_features = vk::PhysicalDeviceVulkan12Features::default()
            .shader_sampled_image_array_non_uniform_indexing(true)
            .descriptor_binding_partially_bound(true)
            .runtime_descriptor_array(true);

        let binding = DEVICE_EXTENSIONS.map(|name| name.as_ptr());
        let create_info = vk::DeviceCreateInfo::default()
            .push_next(&mut vulkan_13_features)
            .push_next(&mut vulkan_12_features)
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_features(&physical_device_features)
            .enabled_extension_names(binding.as_slice());

        let logical_device: ash::Device = unsafe {
            instance
                .create_device(physical_device, &create_info, None)
                .expect("Failed to create device!")
        };

        let graphics_queue =
            unsafe { logical_device.get_device_queue(queue_indices.graphics_queue_index, 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(queue_indices.present_queue_index, 0) };

        let command_pool = Self::create_command_pool(&logical_device, &queue_indices);

        let min_ubo_alignment = unsafe {
            let xc = instance.get_physical_device_properties(physical_device);
            xc.limits.min_uniform_buffer_offset_alignment as u64
        };

        Self {
            logical_device,
            _physical_device: physical_device,
            queue_info: QueueInfo {
                graphics_queue,
                present_queue,
                graphics_queue_index: queue_indices.graphics_queue_index,
                present_queue_index: queue_indices.present_queue_index,
            },
            swapchain_support_details,
            command_pool,
            min_ubo_alignment,
        }
    }

    pub fn update_swapchain_capabilities(&mut self, surface_info: &SurfaceInfo) {
        self.swapchain_support_details =
            Self::query_swap_chain_support(self._physical_device, surface_info);
    }

    fn pick_physical_device(
        instance: &ash::Instance,
        surface_info: &SurfaceInfo,
    ) -> vk::PhysicalDevice {
        let physical_devices: Vec<vk::PhysicalDevice> = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices!")
        };

        println!(
            "{} devices (GPU) found with vulkan support.",
            physical_devices.len()
        );

        let mut result = None;
        for &physical_device in physical_devices.iter() {
            let swapchain_support_details =
                Self::query_swap_chain_support(physical_device, surface_info);
            if Self::is_physical_device_suitable(
                instance,
                physical_device,
                surface_info,
                &swapchain_support_details,
            ) && result.is_none()
            {
                result = Some(physical_device);
                break;
            }
        }

        match result {
            None => panic!("Failed to find a suitable GPU!"),
            Some(physical_device) => physical_device,
        }
    }

    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &SurfaceInfo,
        swapchain_support_details: &SwapChainSupportDetails,
    ) -> bool {
        let indices = Self::find_queue_family(instance, physical_device, surface_info);
        let extensions_supported = Self::check_device_extension_support(instance, physical_device);

        let mut swapchain_adequate = false;
        if extensions_supported {
            swapchain_adequate = !swapchain_support_details.formats.is_empty()
                && !swapchain_support_details.present_modes.is_empty();
        }

        indices.is_some() && extensions_supported && swapchain_adequate
    }

    fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &SurfaceInfo,
    ) -> Option<QueueFamiliyIndices> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut graphics_queue_index = None;
        let mut present_queue_index = None;

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_queue_index = Some(i as u32);
            }

            let is_present_support = unsafe {
                surface_info
                    .surface_instance
                    .get_physical_device_surface_support(
                        physical_device,
                        i as u32,
                        surface_info.surface,
                    )
            };
            if queue_family.queue_count > 0 && is_present_support.unwrap() {
                present_queue_index = Some(i as u32);
            }

            if graphics_queue_index.is_some() && present_queue_index.is_some() {
                break;
            }
        }

        if graphics_queue_index.is_none() || present_queue_index.is_none() {
            return None;
        }

        Some(QueueFamiliyIndices {
            graphics_queue_index: graphics_queue_index.unwrap(),
            present_queue_index: present_queue_index.unwrap(),
        })
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        let extensions = unsafe { instance.enumerate_device_extension_properties(physical_device) };

        match extensions {
            Ok(extensions) => DEVICE_EXTENSIONS.iter().all(|extension| {
                extensions
                    .iter()
                    .any(|ex| extension == &ex.extension_name_as_c_str().unwrap())
            }),
            _ => false,
        }
    }

    fn query_swap_chain_support(
        physical_device: vk::PhysicalDevice,
        surface_info: &SurfaceInfo,
    ) -> SwapChainSupportDetails {
        let surface_capabilities = unsafe {
            surface_info
                .surface_instance
                .get_physical_device_surface_capabilities(physical_device, surface_info.surface)
                .expect("Error queue")
        };

        let formats = unsafe {
            surface_info
                .surface_instance
                .get_physical_device_surface_formats(physical_device, surface_info.surface)
                .expect("Error queue")
        };

        let present_modes = unsafe {
            surface_info
                .surface_instance
                .get_physical_device_surface_present_modes(physical_device, surface_info.surface)
                .expect("Error queue")
        };

        SwapChainSupportDetails {
            capabilies: surface_capabilities,
            formats,
            present_modes,
        }
    }

    fn create_command_pool(
        logical_device: &ash::Device,
        queue_family_indeces: &QueueFamiliyIndices,
    ) -> ash::vk::CommandPool {
        let command_pool_create_info = ash::vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_indeces.graphics_queue_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        unsafe {
            logical_device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Unable to create command pool")
        }
    }
}

pub struct QueueInfo {
    pub graphics_queue_index: u32,
    pub present_queue_index: u32,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

#[derive(Default)]
struct QueueFamiliyIndices {
    graphics_queue_index: u32,
    present_queue_index: u32,
}

pub struct SwapChainSupportDetails {
    pub capabilies: ash::vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<ash::vk::SurfaceFormatKHR>,
    pub present_modes: Vec<ash::vk::PresentModeKHR>,
}
