use ash::vk;

pub trait Render {
    fn render(&self, command_buffer: vk::CommandBuffer);
}