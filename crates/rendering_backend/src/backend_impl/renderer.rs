use crate::vulkan_backend::VulkanBackend;

pub trait Renderer {
    fn new(vulkan_backend: &VulkanBackend) -> Self;
    
    fn draw(&mut self, vulkan_backend: &VulkanBackend);
}
