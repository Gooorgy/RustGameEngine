use std::ptr;

use ash::{khr, vk};
use super::{constants, device, surface::SurfaceInfo};

pub struct SwapchainInfo {
    pub swapchain_device: khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,

    pub swapchain_image_format: vk::SurfaceFormatKHR,
    pub swapchain_extent: vk::Extent2D,
}

impl SwapchainInfo {
    pub fn new(
        instance: &ash::Instance,
        device_info: &device::DeviceInfo,
        surface_info: &SurfaceInfo,
    ) -> SwapchainInfo {
        let surface_format =
            Self::choose_swapchain_format(&device_info.swapchain_support_details.formats);
        let present_mode =
            Self::choose_swap_present_mode(&device_info.swapchain_support_details.present_modes);
        let extent = Self::chosse_swap_extent(&device_info.swapchain_support_details.capabilies);

        let mut image_count = device_info
            .swapchain_support_details
            .capabilies
            .min_image_count
            + 1;

        if device_info
            .swapchain_support_details
            .capabilies
            .max_image_count
            > 0
            && image_count
                > device_info
                    .swapchain_support_details
                    .capabilies
                    .max_image_count
        {
            image_count = device_info
                .swapchain_support_details
                .capabilies
                .max_image_count;
        }

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: surface_info.surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            pre_transform: device_info
                .swapchain_support_details
                .capabilies
                .current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            ..Default::default()
        };

        let indices = [
            device_info.queue_info.graphics_queue_index,
            device_info.queue_info.present_queue_index,
        ];

        if device_info.queue_info.graphics_queue_index != device_info.queue_info.present_queue_index
        {
            swapchain_create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            swapchain_create_info.queue_family_index_count = 2;
            swapchain_create_info.p_queue_family_indices = indices.as_ptr();
        } else {
            swapchain_create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
            swapchain_create_info.queue_family_index_count = 0;
            swapchain_create_info.p_queue_family_indices = ptr::null();
        }

        let swapchain_device =
            ash::khr::swapchain::Device::new(instance, &device_info.logical_device);
        let swapchain = unsafe {
            swapchain_device
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create swapchain!")
        };

        let swapchain_images = unsafe { swapchain_device.get_swapchain_images(swapchain) };

        SwapchainInfo {
            swapchain_device,
            swapchain,
            swapchain_images: swapchain_images.unwrap(),
            swapchain_image_format: surface_format,
            swapchain_extent: extent,
        }
    }

    fn choose_swapchain_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        for &format in available_formats.iter() {
            if format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return format;
            }
        }

        return *available_formats.first().unwrap();
    }

    fn choose_swap_present_mode(
        available_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        for &present_mode in available_present_modes.iter() {
            if present_mode == vk::PresentModeKHR::MAILBOX {
                return present_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    fn chosse_swap_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: num::clamp(
                    constants::WINDOW_WIDTH,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: num::clamp(
                    constants::WINDOW_HEIGHT,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}
