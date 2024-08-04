pub mod structs;
pub mod utils;

mod graphics_pipeline;
mod render_pass;
mod swapchain;

use crate::swapchain::structs::SwapchainInfo;
use ash::vk::PipelineLayout;
use ash::{vk, Entry, Instance};
use new::frame_buffer;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::ptr;
use structs::{QueueFamiliyIndices, SurfaceInfo, SwapChainSupportDetails};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;
use winit::{application::ApplicationHandler, dpi::LogicalSize};

const WINDOW_TITLE: &str = "Vulkan Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

const DEVICE_EXTENSIONS: [&CStr; 1] = [vk::KHR_SWAPCHAIN_NAME];

struct VulkanApp {
    _entry: Entry,
    _instance: Instance,
    _physical_device: vk::PhysicalDevice,
    _device: ash::Device,
    _graphics_queue: vk::Queue,
    _present_queue: vk::Queue,
    _surface_info: SurfaceInfo,
    _swapchain_info: SwapchainInfo,
    _image_views: Vec<vk::ImageView>,
    _pipeline_layout: PipelineLayout,
    _render_pass: vk::RenderPass,
    _graphics_pipeline: vk::Pipeline,
    _swapchain_frame_buffer: Vec<vk::Framebuffer>,
}

#[derive(Default)]
struct AppWindow {
    window: Option<winit::window::Window>,
}

impl VulkanApp {
    fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { Entry::load()? };
        let instance = Self::create_instance(&entry, window);
        let surface_info = Self::create_surface(&entry, &instance, window);
        let physical_device = Self::pick_physical_device(&instance, &surface_info);
        let (device, family_indices) =
            Self::create_logical_device(&instance, physical_device, &surface_info);
        let swapchain_info =
            Self::create_swapchain(&instance, physical_device, &surface_info, &device);

        let image_views = Self::create_image_views(&swapchain_info, &device);
        let graphics_queue =
            unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

        let render_pass = render_pass::render_pass::create(&swapchain_info, &device);
        let (pipeline_layout, graphics_pipeline) =
            graphics_pipeline::graphics_pipeline::create(&device, &render_pass);

        let frame_buffers = frame_buffer::frame_buffer::create_buffers(
            &device,
            &image_views,
            &render_pass,
            &swapchain_info.swapchain_extent,
        );

        Ok(Self {
            _entry: entry,
            _instance: instance,
            _physical_device: physical_device,
            _device: device,
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
            _surface_info: surface_info,
            _swapchain_info: swapchain_info,
            _image_views: image_views,
            _pipeline_layout: pipeline_layout,
            _render_pass: render_pass,
            _graphics_pipeline: graphics_pipeline,
            _swapchain_frame_buffer: frame_buffers,
        })
    }

    fn create_instance(entry: &Entry, window: &Window) -> Instance {
        let app_name = CString::new("Vulkan Application").unwrap();
        let engine_name = CString::new("No Engine").unwrap();

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name.as_c_str())
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(engine_name.as_c_str())
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap();
        let extension_names = extension_names.to_vec();

        let instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn pick_physical_device(instance: &Instance, surface_info: &SurfaceInfo) -> vk::PhysicalDevice {
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
            if VulkanApp::is_physical_device_suitable(instance, physical_device, surface_info) {
                if result.is_none() {
                    result = Some(physical_device);
                    break;
                }
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
    ) -> bool {
        let indices = VulkanApp::find_queue_family(instance, physical_device, surface_info);

        let extensions_supported =
            VulkanApp::check_device_extension_support(instance, physical_device);

        let mut swapchain_adequate = false;
        if extensions_supported {
            let swapchain_support_details =
                VulkanApp::query_swap_chain_support(physical_device, surface_info);
            swapchain_adequate = !swapchain_support_details.formats.is_empty()
                && !swapchain_support_details.present_modes.is_empty();
        }

        return indices.is_complete() && extensions_supported && swapchain_adequate;
    }

    fn check_device_extension_support(
        instance: &Instance,
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

    fn find_queue_family(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &SurfaceInfo,
    ) -> structs::QueueFamiliyIndices {
        let mut indices = QueueFamiliyIndices {
            graphics_family: None,
            present_family: None,
        };
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
            }

            let is_present_support = unsafe {
                surface_info
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        i as u32,
                        surface_info.surface,
                    )
            };
            if queue_family.queue_count > 0 && is_present_support.unwrap() {
                indices.present_family = Some(i as u32);
            }

            if indices.is_complete() {
                break;
            }
        }

        return indices;
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &SurfaceInfo,
    ) -> (ash::Device, QueueFamiliyIndices) {
        let queue_family_indices =
            VulkanApp::find_queue_family(instance, physical_device, surface_info);

        use std::collections::HashSet;
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(queue_family_indices.graphics_family.unwrap());
        unique_queue_families.insert(queue_family_indices.present_family.unwrap());

        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];

        for &unique_queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                queue_family_index: unique_queue_family,
                queue_count: 1,
                p_queue_priorities: queue_priorities.as_ptr(),
                ..Default::default()
            };

            queue_create_infos.push(queue_create_info)
        }

        let physical_device_features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            p_queue_create_infos: queue_create_infos.as_ptr(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_enabled_features: &physical_device_features,
            pp_enabled_extension_names: DEVICE_EXTENSIONS.map(|name| name.as_ptr()).as_ptr(),
            enabled_extension_count: DEVICE_EXTENSIONS.len() as u32,

            ..Default::default()
        };

        let device = match unsafe { instance.create_device(physical_device, &create_info, None) } {
            Ok(device) => device,
            Err(_e) => panic!("Failed to create logical device!"),
        };

        (device, queue_family_indices)
    }

    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> SurfaceInfo {
        let window_handle = window.window_handle().unwrap().as_raw();
        let display_handle = window.display_handle().unwrap().as_raw();
        let surface = unsafe {
            ash_window::create_surface(entry, instance, display_handle, window_handle, None)
                .unwrap()
        };

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        SurfaceInfo {
            surface,
            surface_loader,
        }
    }

    fn query_swap_chain_support(
        physical_device: vk::PhysicalDevice,
        surface_info: &SurfaceInfo,
    ) -> SwapChainSupportDetails {
        let surface_capabilities = unsafe {
            surface_info
                .surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface_info.surface)
                .unwrap()
        };

        let formats = unsafe {
            surface_info
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface_info.surface)
                .unwrap()
        };

        let present_modes = unsafe {
            surface_info
                .surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface_info.surface)
                .unwrap()
        };

        SwapChainSupportDetails {
            capabilies: surface_capabilities,
            formats,
            present_modes,
        }
    }

    fn choose_swapchain_format(
        available_formats: Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        for &format in available_formats.iter() {
            if format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return format;
            }
        }

        return available_formats.first().unwrap().clone();
    }

    fn choose_swap_present_mode(
        available_present_modes: Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for &present_mode in available_present_modes.iter() {
            if present_mode == vk::PresentModeKHR::MAILBOX {
                return present_mode;
            }
        }

        return vk::PresentModeKHR::FIFO;
    }

    fn chosse_swap_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: num::clamp(
                    WINDOW_WIDTH,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: num::clamp(
                    WINDOW_HEIGHT,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    fn create_swapchain(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &SurfaceInfo,
        device: &ash::Device,
    ) -> SwapchainInfo {
        let swapchain_support_details =
            VulkanApp::query_swap_chain_support(physical_device, surface_info);

        let surface_format = VulkanApp::choose_swapchain_format(swapchain_support_details.formats);
        let present_mode =
            VulkanApp::choose_swap_present_mode(swapchain_support_details.present_modes);
        let extent = VulkanApp::chosse_swap_extent(&swapchain_support_details.capabilies);

        let mut image_count = swapchain_support_details.capabilies.min_image_count + 1;

        if swapchain_support_details.capabilies.max_image_count > 0
            && image_count > swapchain_support_details.capabilies.max_image_count
        {
            image_count = swapchain_support_details.capabilies.max_image_count;
        }

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: surface_info.surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            pre_transform: swapchain_support_details.capabilies.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            ..Default::default()
        };

        let queue_familiy_indices =
            VulkanApp::find_queue_family(instance, physical_device, surface_info);
        let indices = [
            queue_familiy_indices.graphics_family.unwrap(),
            queue_familiy_indices.present_family.unwrap(),
        ];

        if queue_familiy_indices.graphics_family != queue_familiy_indices.present_family {
            swapchain_create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            swapchain_create_info.queue_family_index_count = 2;
            swapchain_create_info.p_queue_family_indices = indices.as_ptr();
        } else {
            swapchain_create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
            swapchain_create_info.queue_family_index_count = 0;
            swapchain_create_info.p_queue_family_indices = ptr::null();
        }

        let swapchain_device = ash::khr::swapchain::Device::new(instance, device);
        let swapchain_result =
            unsafe { swapchain_device.create_swapchain(&swapchain_create_info, None) };

        let swapchain = match swapchain_result {
            Ok(swapchain) => swapchain,
            Err(_) => panic!("Error creating swapchain"),
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

    fn create_image_views(
        swapchain_info: &SwapchainInfo,
        device: &ash::Device,
    ) -> Vec<vk::ImageView> {
        let mut image_views = vec![];

        for swapchain_image in swapchain_info.swapchain_images.clone() {
            let image_view_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: swapchain_image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: swapchain_info.swapchain_image_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };

            let image_view_result =
                unsafe { device.create_image_view(&image_view_create_info, None) };

            let image_view = match image_view_result {
                Ok(image_view) => image_view,
                _ => panic!("Error creating image view"),
            };

            image_views.push(image_view);
        }

        image_views
    }
}

impl ApplicationHandler for AppWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        self.window = Some(event_loop.create_window(window_attributes).unwrap());

        let _x: Result<VulkanApp, Box<dyn Error>> = VulkanApp::new(&self.window.as_ref().unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app_window = AppWindow::default();
    let _ = event_loop.run_app(&mut app_window);
}
