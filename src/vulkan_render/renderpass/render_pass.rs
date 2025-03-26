use crate::vulkan_render::device::DeviceInfo;

pub trait RenderPass {
    fn init(
        &mut self,
        device_info: DeviceInfo,
    );
    fn update(&mut self, current_frame: usize);
    fn cleanup(&mut self, device_info: DeviceInfo);
}