pub fn create_buffers(
    device: &ash::Device,
    image_views: &Vec<ash::vk::ImageView>,
    render_pass: &ash::vk::RenderPass,
    swapchain_extend: &ash::vk::Extent2D,
) -> Vec<ash::vk::Framebuffer> {
    let mut frame_buffers = vec![];

    for image_view in image_views {
        let frame_buffer_create_info = ash::vk::FramebufferCreateInfo {
            s_type: ash::vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            render_pass: *render_pass,
            attachment_count: 1,
            p_attachments: image_view,
            width: swapchain_extend.width,
            height: swapchain_extend.height,
            layers: 1,
            ..Default::default()
        };

        let buffer = unsafe {
            device
                .create_framebuffer(&frame_buffer_create_info, None)
                .expect("Unable to create frame buffer")
        };
        frame_buffers.push(buffer);
    }

    frame_buffers
}
