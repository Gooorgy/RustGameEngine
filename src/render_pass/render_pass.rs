use crate::{graphics_pipeline, swapchain::structs::SwapchainInfo};

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

    let subpass_dependency = ash::vk::SubpassDependency {
        src_subpass: ash::vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: ash::vk::AccessFlags::empty(),
        dst_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        ..Default::default()
    };

    let render_pass_create_info = ash::vk::RenderPassCreateInfo {
        s_type: ash::vk::StructureType::RENDER_PASS_CREATE_INFO,
        attachment_count: 1,
        p_attachments: &color_attachment,
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 1,
        p_dependencies: &subpass_dependency,
        ..Default::default()
    };

    unsafe {
        device
            .create_render_pass(&render_pass_create_info, None)
            .expect("Unable to create render pass")
    }
}

pub fn begin_render_pass(
    device: &ash::Device,
    render_pass: &ash::vk::RenderPass,
    swapchain_frame_buffers: &Vec<ash::vk::Framebuffer>,
    command_buffers: &Vec<ash::vk::CommandBuffer>,
    graphics_pipeline: ash::vk::Pipeline,
    extent: &ash::vk::Extent2D,
) {
    let clear_values = [ash::vk::ClearValue {
        color: ash::vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    }];

    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let render_pass_begin_info = ash::vk::RenderPassBeginInfo {
            s_type: ash::vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: *render_pass,
            framebuffer: swapchain_frame_buffers[i],
            render_area: ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: *extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                ash::vk::SubpassContents::INLINE,
            );

            device.cmd_bind_pipeline(
                command_buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );

            let viewport = ash::vk::Viewport {
                x: 0.0f32,
                y: 0.0f32,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0 as f32,
                max_depth: 1 as f32,
                ..Default::default()
            };

            device.cmd_set_viewport(command_buffer, 0, &[viewport]);

            let scissor = ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: *extent,
                ..Default::default()
            };

            device.cmd_set_scissor(command_buffer, 0, &[scissor]);

            device.cmd_draw(command_buffer, 3, 1, 0, 0);

            device.cmd_end_render_pass(command_buffer);

            device.end_command_buffer(command_buffer);
        }
    }
}
