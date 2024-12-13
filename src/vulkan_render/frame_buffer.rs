use ash::vk;

pub fn create_buffers(
    logical_device: &ash::Device,
    image_views: &Vec<ash::vk::ImageView>,
    depth_image_view: vk::ImageView,
    render_pass: &ash::vk::RenderPass,
    swapchain_extend: &ash::vk::Extent2D,
) -> Vec<ash::vk::Framebuffer> {
    let mut frame_buffers = vec![];

    for image_view in image_views {
        let image_view = &[*image_view, depth_image_view];
        let frame_buffer_create_info = ash::vk::FramebufferCreateInfo::default()
            .render_pass(*render_pass)
            .attachments(image_view)
            .width(swapchain_extend.width)
            .layers(1)
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
