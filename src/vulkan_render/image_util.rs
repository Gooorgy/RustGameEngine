use ash::vk;
use ash::vk::ImageSubresourceLayers;
use crate::vulkan_render::buffer::BufferInfo;
use crate::vulkan_render::device::DeviceInfo;

pub fn copy_image_to_image(
    device: &ash::Device,
    command_buffer: &vk::CommandBuffer,
    src_image: vk::Image,
    dst_image: vk::Image,
    src_size: vk::Extent2D,
    dst_size: vk::Extent2D
) {
    let mut blit_region = vk::ImageBlit2::default()
        .src_subresource(ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            layer_count: 1,
            base_array_layer: 0,
            mip_level: 0,
        })
        .dst_subresource(ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            layer_count: 1,
            base_array_layer: 0,
            mip_level: 0,
        });

    blit_region.src_offsets[1].x = src_size.width as i32;
    blit_region.src_offsets[1].y = src_size.height as i32;
    blit_region.src_offsets[1].z = 1;

    blit_region.dst_offsets[1].x = dst_size.width as i32;
    blit_region.dst_offsets[1].y = dst_size.height as i32;
    blit_region.dst_offsets[1].z = 1;

    let regions = [blit_region];

    let blit_info = vk::BlitImageInfo2::default()
        .src_image(src_image)
        .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .dst_image(dst_image)
        .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .filter(vk::Filter::LINEAR)
        .regions(&regions);

    unsafe {
        device.cmd_blit_image2(*command_buffer,&blit_info)
    }
}

pub fn transition_image_layout(
    device_info: &DeviceInfo,
    command_buffer: &vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    depth: bool
) {
    //let command_buffer = BufferInfo::begin_single_time_command(device_info);

    let mut aspect_mask = vk::ImageAspectFlags::COLOR;
    if(depth) {
        aspect_mask = vk::ImageAspectFlags::DEPTH;
    }

    let mut src_access_mask;
    let mut dst_access_mask;
    let mut source_stage;
    let mut destination_stage;

    if old_layout == vk::ImageLayout::UNDEFINED
        && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
    {
        src_access_mask = vk::AccessFlags::empty();
        dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

        source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        destination_stage = vk::PipelineStageFlags::TRANSFER;
    } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        && new_layout == ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    {
        src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        dst_access_mask = vk::AccessFlags::SHADER_READ;

        source_stage = vk::PipelineStageFlags::TRANSFER;
        destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    }

    src_access_mask = vk::AccessFlags::MEMORY_WRITE;
    dst_access_mask = vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE;

    source_stage = vk::PipelineStageFlags::ALL_COMMANDS;
    destination_stage = vk::PipelineStageFlags::ALL_COMMANDS;

    let barrier = vk::ImageMemoryBarrier::default()
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(aspect_mask)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    unsafe {
        device_info.logical_device.cmd_pipeline_barrier(
            *command_buffer,
            source_stage,
            destination_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    }

    //BufferInfo::end_single_time_command(device_info, command_buffer);
}