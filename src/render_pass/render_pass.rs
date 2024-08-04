use crate::swapchain::structs::SwapchainInfo;

pub fn create(swapchain_info: &SwapchainInfo, device: &ash::Device) -> ash::vk::RenderPass {
    let color_attachment = ash::vk::AttachmentDescription {
        format: swapchain_info.swapchain_image_format.format,
        samples: ash::vk::SampleCountFlags::TYPE_1,
        load_op: ash::vk::AttachmentLoadOp::CLEAR,
        store_op: ash::vk::AttachmentStoreOp::STORE,
        stencil_load_op: ash::vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: ash::vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: ash::vk::ImageLayout::UNDEFINED,
        final_layout: ash::vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    };

    let color_attachment_ref = ash::vk::AttachmentReference {
        attachment: 0,
        layout: ash::vk::ImageLayout::ATTACHMENT_OPTIMAL,
    };

    let subpass = ash::vk::SubpassDescription {
        pipeline_bind_point: ash::vk::PipelineBindPoint::GRAPHICS,
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        ..Default::default()
    };

    let render_pass_create_info = ash::vk::RenderPassCreateInfo {
        s_type: ash::vk::StructureType::RENDER_PASS_CREATE_INFO,
        attachment_count: 1,
        p_attachments: &color_attachment,
        subpass_count: 1,
        p_subpasses: &subpass,
        ..Default::default()
    };

    unsafe {
        device
            .create_render_pass(&render_pass_create_info, None)
            .expect("Unable to create render pass")
    }
}
