pub struct QueueFamiliyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamiliyIndices {
    pub fn is_complete(&self) -> bool {
        return self.graphics_family.is_some() && self.present_family.is_some();
    }
}

pub struct SurfaceInfo {
    pub surface_loader: ash::khr::surface::Instance,
    pub surface: ash::vk::SurfaceKHR,
}

pub struct SwapchainInfo {
    pub swapchain_device: ash::khr::swapchain::Device,
    pub swapchain: ash::vk::SwapchainKHR,
    pub swapchain_images: Vec<ash::vk::Image>,

    pub swapchain_image_format: ash::vk::SurfaceFormatKHR,
    pub swapchain_extent: ash::vk::Extent2D,
}

pub struct SwapChainSupportDetails {
    pub capabilies: ash::vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<ash::vk::SurfaceFormatKHR>,
    pub present_modes: Vec<ash::vk::PresentModeKHR>,
}
