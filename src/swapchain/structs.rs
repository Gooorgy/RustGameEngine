pub struct SwapchainInfo {
    pub swapchain_device: ash::khr::swapchain::Device,
    pub swapchain: ash::vk::SwapchainKHR,
    pub swapchain_images: Vec<ash::vk::Image>,

    pub swapchain_image_format: ash::vk::SurfaceFormatKHR,
    pub swapchain_extent: ash::vk::Extent2D,
}
