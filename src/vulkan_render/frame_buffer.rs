pub fn create_buffers(
    logical_device: &ash::Device,
    image_views: &Vec<ash::vk::ImageView>,
    render_pass: &ash::vk::RenderPass,
    swapchain_extend: &ash::vk::Extent2D,
) -> Vec<ash::vk::Framebuffer> {
    let mut frame_buffers = vec![];

    for image_view in image_views {
        let image_view = [*image_view];
        let frame_buffer_create_info = ash::vk::FramebufferCreateInfo::default()
            .render_pass(*render_pass)
            .attachment_count(1)
            .attachments(&image_view)
            .width(swapchain_extend.width)
            .height(swapchain_extend.height);

        let buffer = unsafe {
            logical_device
                .create_framebuffer(&frame_buffer_create_info, None)
                .expect("Unable to create frame buffer")
        };
        frame_buffers.push(buffer);
    }

    frame_buffers
}
