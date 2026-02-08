use crate::backend_impl::device::DeviceInfo;
use crate::backend_impl::utils;
use crate::image::{ImageAspect, ImageDesc, ImageUsageFlags, TextureFormat};
use ash::{vk, Device, Instance};

pub struct AllocatedImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_memory: vk::DeviceMemory,
    pub image_extent: vk::Extent3D,
    pub image_format: vk::Format,
    pub image_layout: vk::ImageLayout,
}

pub enum AllocatedImageType {
    Color,
    Depth,
}

impl AllocatedImage {
    pub fn new(
        image_desc: ImageDesc,
        device_info: &DeviceInfo,
        instance: &Instance,
        mem_properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let extent = vk::Extent3D {
            width: image_desc.width,
            height: image_desc.height,
            depth: image_desc.depth,
        };

        let format = map_texture_format(image_desc.format);
        let aspect_flags = map_aspect(image_desc.aspect);
        let usage_flags = map_usage_flags(image_desc.usage);

        let image = Self::create_image(
            &device_info.logical_device,
            format,
            vk::ImageTiling::OPTIMAL,
            usage_flags,
            extent,
        );
        let image_memory = Self::allocate_image(device_info, instance, &image, mem_properties);
        let image_view = Self::create_image_view(device_info, &image, format, aspect_flags);

        Self {
            image,
            image_view,
            image_memory,
            image_format: format,
            image_extent: extent,
            image_layout: vk::ImageLayout::UNDEFINED,
        }
    }

    pub fn create_image(
        device: &Device,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        extent: vk::Extent3D,
    ) -> vk::Image {
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1)
            .flags(vk::ImageCreateFlags::empty());

        unsafe {
            device
                .create_image(&image_create_info, None)
                .expect("failed to create image")
        }
    }

    fn allocate_image(
        device_info: &DeviceInfo,
        instance: &Instance,
        image: &vk::Image,
        mem_properties: vk::MemoryPropertyFlags,
    ) -> vk::DeviceMemory {
        let mem_requirements = unsafe {
            device_info
                .logical_device
                .get_image_memory_requirements(*image)
        };

        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device_info._physical_device) };

        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(utils::find_memory_type(
                mem_requirements.memory_type_bits,
                mem_properties,
                memory_properties,
            ));

        let allocated_memory = unsafe {
            device_info
                .logical_device
                .allocate_memory(&allocate_info, None)
                .expect("failed to allocate image memory")
        };

        unsafe {
            device_info
                .logical_device
                .bind_image_memory(*image, allocated_memory, 0)
                .expect("failed to bind image memory");
        }

        allocated_memory
    }

    pub fn create_image_view(
        device_info: &DeviceInfo,
        image: &vk::Image,
        format: vk::Format,
        image_aspect_flags: vk::ImageAspectFlags,
    ) -> vk::ImageView {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(*image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(image_aspect_flags)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        unsafe {
            device_info
                .logical_device
                .create_image_view(&view_info, None)
                .expect("failed to create image view")
        }
    }
}

pub fn copy_image_to_image(
    device: &Device,
    command_buffer: &vk::CommandBuffer,
    src_image: vk::Image,
    dst_image: vk::Image,
    src_size: vk::Extent2D,
    dst_size: vk::Extent2D,
) {
    let mut blit_region = vk::ImageBlit2::default()
        .src_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            layer_count: 1,
            base_array_layer: 0,
            mip_level: 0,
        })
        .dst_subresource(vk::ImageSubresourceLayers {
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

    unsafe { device.cmd_blit_image2(*command_buffer, &blit_info) }
}

pub fn transition_image_layout(
    device_info: &DeviceInfo,
    command_buffer: &vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    depth: bool,
) {
    let mut aspect_mask = vk::ImageAspectFlags::COLOR;
    if depth {
        aspect_mask = vk::ImageAspectFlags::DEPTH;
    }

    let src_access_mask;
    let dst_access_mask;
    let source_stage;
    let destination_stage;

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
}

pub fn create_image(
    device_info: &DeviceInfo,
    instance: &Instance,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    mem_properties: vk::MemoryPropertyFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            height,
            width,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1)
        .flags(vk::ImageCreateFlags::empty());

    let x = unsafe {
        device_info
            .logical_device
            .create_image(&image_info, None)
            .expect("failed to create image")
    };

    let mem_requirements = unsafe { device_info.logical_device.get_image_memory_requirements(x) };

    let memory_properties =
        unsafe { instance.get_physical_device_memory_properties(device_info._physical_device) };

    let allocate_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(utils::find_memory_type(
            mem_requirements.memory_type_bits,
            mem_properties,
            memory_properties,
        ));

    let allocated_memory = unsafe {
        device_info
            .logical_device
            .allocate_memory(&allocate_info, None)
            .expect("failed to allocate image memory")
    };

    unsafe {
        device_info
            .logical_device
            .bind_image_memory(x, allocated_memory, 0)
            .expect("failed to bind image memory");
    }

    return (x, allocated_memory);
}

fn map_texture_format(texture_format: TextureFormat) -> vk::Format {
    match texture_format {
        TextureFormat::R8g8b8a8Unorm => vk::Format::R8G8B8A8_UNORM,
        TextureFormat::D32Float => vk::Format::D32_SFLOAT,
        TextureFormat::R16g16b16a16Float => vk::Format::R16G16B16A16_SFLOAT,
    }
}

fn map_aspect(aspect: ImageAspect) -> vk::ImageAspectFlags {
    match aspect {
        ImageAspect::Color => vk::ImageAspectFlags::COLOR,
        ImageAspect::Depth => vk::ImageAspectFlags::DEPTH,
        ImageAspect::Stencil => vk::ImageAspectFlags::STENCIL,
        ImageAspect::DepthStencil => {
            let mut flags = vk::ImageAspectFlags::empty();
            flags |= vk::ImageAspectFlags::DEPTH;
            flags |= vk::ImageAspectFlags::STENCIL;
            flags
        }
    }
}

fn map_usage_flags(usage: ImageUsageFlags) -> vk::ImageUsageFlags {
    let mut flags = vk::ImageUsageFlags::empty();
    if usage.contains(ImageUsageFlags::COLOR_ATTACHMENT) {
        flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
    }
    if usage.contains(ImageUsageFlags::DEPTH_ATTACHMENT) {
        flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
    }
    if usage.contains(ImageUsageFlags::SAMPLED) {
        flags |= vk::ImageUsageFlags::SAMPLED;
    }
    if usage.contains(ImageUsageFlags::STORAGE) {
        flags |= vk::ImageUsageFlags::STORAGE;
    }
    if usage.contains(ImageUsageFlags::TRANSFER_SRC) {
        flags |= vk::ImageUsageFlags::TRANSFER_SRC;
    }
    if usage.contains(ImageUsageFlags::TRANSFER_DST) {
        flags |= vk::ImageUsageFlags::TRANSFER_DST;
    }
    flags
}
