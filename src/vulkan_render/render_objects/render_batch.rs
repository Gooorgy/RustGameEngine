use crate::vulkan_render::device::DeviceInfo;
use crate::vulkan_render::graphics_pipeline::PipelineInfo;
use crate::vulkan_render::render_objects::draw_objects::Drawable;
use ash::vk;

pub struct RenderBatch {
    pub pipeline: PipelineInfo,
    pub drawables: Vec<Drawable>,
}

pub trait Draw {
    fn draw(&self, command_buffer: vk::CommandBuffer, device_info: &DeviceInfo);
}
