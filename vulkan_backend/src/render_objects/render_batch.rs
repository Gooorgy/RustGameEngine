use ash::vk;
use crate::device::DeviceInfo;
use crate::graphics_pipeline::PipelineInfo;
use crate::render_objects::draw_objects::Drawable;

pub struct RenderBatch {
    pub pipeline: PipelineInfo,
    pub drawables: Vec<Drawable>,
}

pub trait Draw {
    fn draw(&self, command_buffer: vk::CommandBuffer, device_info: &DeviceInfo);
}
