use core::slice;
use std::{
    error::Error,
    ffi::{c_void, CString},
    mem, ptr,
};

use ash::{
    vk::{self},
    Instance,
};
use std::time::Instant;

use ash::vk::{
    CommandBuffer, DescriptorSet, DeviceMemory, Extent3D, Image, ImageAspectFlags,
    ImageSubresourceRange,
};
use cgmath::{assert_abs_diff_eq, Matrix4, Point3, Vector3};
use image::{DynamicImage, GenericImageView, ImageReader};
use winit::{raw_window_handle::HasDisplayHandle, window::Window};

use crate::{INDICES, VERTICES};

use super::{
    buffer::{self, BufferInfo},
    command_buffer::CommandBufferInfo,
    constants, descriptors,
    device::{self, DeviceInfo},
    frame_buffer,
    graphics_pipeline::PipelineInfo,
    structs::{UniformBufferObject, Vertex},
    surface::{self, SurfaceInfo},
    swapchain::{self, SwapchainInfo},
    validation,
};

pub struct VulkanBackend {
    _entry: ash::Entry,
    instance: ash::Instance,
    device_info: DeviceInfo,
    surface_info: SurfaceInfo,
    swapchain_info: SwapchainInfo,
    image_views: Vec<vk::ImageView>,
    pipeline_info: PipelineInfo,
    render_pass: vk::RenderPass,
    swapchain_frame_buffers: Vec<vk::Framebuffer>,
    image_available_semaphores: Vec<ash::vk::Semaphore>,
    render_finished_semaphores: Vec<ash::vk::Semaphore>,
    in_flight_fences: Vec<ash::vk::Fence>,
    command_buffer_info: CommandBufferInfo,
    current_frame: u32,
    vertex_buffer_info: BufferInfo,
    index_buffer_info: BufferInfo,
    descriptor_set_layout: vk::DescriptorSetLayout,
    uniform_buffers: Vec<BufferInfo>,
    uniform_buffers_mapped: Vec<*mut UniformBufferObject>,
    descriptor_sets: Vec<DescriptorSet>,
    start_time: Instant,
    ubo: UniformBufferObject,
    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
}

impl VulkanBackend {
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { ash::Entry::load()? };
        Self::create_instance(&entry, window);
        let instance = Self::create_instance(&entry, window);
        let surface_info = surface::SurfaceInfo::new(&entry, &instance, window);
        let device_info = device::DeviceInfo::new(&instance, &surface_info);
        let swapchain_info = swapchain::SwapchainInfo::new(&instance, &device_info, &surface_info);

        let image_views = Self::create_image_views(&swapchain_info, &device_info.logical_device);

        let render_pass = Self::create_render_pass(&swapchain_info, &device_info.logical_device);

        let (texture_image, texture_image_memory) =
            Self::create_texture_image(&device_info, &instance);
        let texture_image_view = Self::create_texture_image_view(&device_info, texture_image);
        let texture_sampler = Self::create_texture_sampler(&device_info, &instance);


        let descriptor_set_layout = Self::create_descriptor_set_layout(&device_info);
        let pipeline_info = PipelineInfo::new(
            &render_pass,
            &device_info.logical_device,
            &descriptor_set_layout,
        );

        let frame_buffers = frame_buffer::create_buffers(
            &device_info.logical_device,
            &image_views,
            &render_pass,
            &swapchain_info.swapchain_extent,
        );



        let vertex_buffer_info = Self::create_vertex_buffer(&instance, &device_info);
        let index_buffer_info = Self::create_index_buffer(&instance, &device_info);
        let (uniform_buffers, uniform_buffers_mapped) =
            Self::create_uniform_buffers(&device_info, &instance);

        let descriptor_pool = descriptors::CustomDescriptorPool::new(
            &device_info.logical_device,
            swapchain_info.swapchain_images.len() as u32,
        );

        let descriptor_sets = descriptor_pool.allocate_descriptor(
            descriptor_set_layout,
            &uniform_buffers,
            swapchain_info.swapchain_images.len() as u32,
            texture_image_view,
            texture_sampler
        );

        let command_buffer_info = CommandBufferInfo::new(&device_info);

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
            Self::create_sync_objects(&device_info.logical_device);

        let start_time = Instant::now();

        let aspect_ratio = swapchain_info.swapchain_extent.width as f32
            / swapchain_info.swapchain_extent.height as f32;
        let mut ubo = UniformBufferObject {
            model: Matrix4::from_angle_z(cgmath::Deg(90.0)),
            view: Matrix4::look_at_rh(
                Point3::new(2.0, 2.0, 2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ),
            proj: cgmath::perspective(cgmath::Deg(45.0), aspect_ratio, 0.1, 10.0),
        };

        ubo.proj[1][1] *= -1.0;

        Ok(Self {
            _entry: entry,
            instance,
            device_info,
            surface_info,
            swapchain_info,
            image_views,
            pipeline_info,
            render_pass,
            swapchain_frame_buffers: frame_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            command_buffer_info,
            current_frame: 0,
            vertex_buffer_info,
            index_buffer_info,
            descriptor_set_layout,
            uniform_buffers,
            uniform_buffers_mapped,
            descriptor_sets,
            start_time,
            ubo,
            texture_image,
            texture_image_memory,
            texture_image_view,
            texture_sampler,
        })
    }

    fn create_texture_sampler(device_info: &DeviceInfo, instance: &Instance) -> vk::Sampler {
        let device_properties =
            unsafe { instance.get_physical_device_properties(device_info._physical_device) };

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(device_properties.limits.max_sampler_anisotropy)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);

        unsafe {
            device_info
                .logical_device
                .create_sampler(&sampler_info, None)
                .expect("failed to create sampler")
        }
    }

    fn create_texture_image_view(device_info: &DeviceInfo, image: vk::Image) -> vk::ImageView {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
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

    fn copy_buffy_to_image(
        device_info: &DeviceInfo,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
    ) {
        let command_buffer = BufferInfo::begin_single_time_command(device_info);

        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                height,
                width,
                depth: 1,
            });

        unsafe {
            device_info.logical_device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        BufferInfo::end_single_time_command(device_info, command_buffer);
    }

    fn transition_image_layout(
        device_info: &DeviceInfo,
        image: vk::Image,
        format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let command_buffer = BufferInfo::begin_single_time_command(device_info);


        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;

            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else {
            panic!("unsupported layout transition");
        }

        let barrier = vk::ImageMemoryBarrier::default()
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(
                ImageSubresourceRange::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        unsafe {
            device_info.logical_device.cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        }

        BufferInfo::end_single_time_command(device_info, command_buffer);
    }

    fn create_texture_image(
        device_info: &DeviceInfo,
        instance: &ash::Instance,
    ) -> (Image, DeviceMemory) {
        let mut dyn_image = image::open("E:\\rust\\new\\src\\texture.png").unwrap();
        let image_width = dyn_image.width();
        let image_height = dyn_image.height();

        let image_size =
            (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;

        let image_data = match &dyn_image {
            image::DynamicImage::ImageLuma8(_) | image::DynamicImage::ImageRgb8(_) => {
                dyn_image.to_rgba8().into_raw()
            }
            _ => vec![],
        };

        let image_buffer = BufferInfo::new(
            instance,
            device_info,
            image_size,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::BufferUsageFlags::TRANSFER_SRC,
        );
        let image_buffer_mapped = unsafe {
            device_info
                .logical_device
                .map_memory(
                    image_buffer.buffer_memory,
                    0,
                    image_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map image buffer memory") as *mut u8
        };

        unsafe {
            image_buffer_mapped.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());

            device_info
                .logical_device
                .unmap_memory(image_buffer.buffer_memory);
        }

        let (image, device_memory) = Self::create_image(device_info, instance, dyn_image);

        Self::transition_image_layout(
            device_info,
            image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        Self::copy_buffy_to_image(
            device_info,
            image_buffer.buffer,
            image,
            image_width,
            image_height,
        );
        Self::transition_image_layout(
            device_info,
            image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );

        unsafe {
            device_info.logical_device.destroy_buffer(image_buffer.buffer, None);
            device_info.logical_device.free_memory(image_buffer.buffer_memory, None);
        }

        (image, device_memory)
    }

    fn create_image(
        device_info: &DeviceInfo,
        instance: &Instance,
        mut image: DynamicImage,
    ) -> (vk::Image, vk::DeviceMemory) {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(Extent3D {
                height: image.height(),
                width: image.width(),
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1)
            .flags(vk::ImageCreateFlags::empty());

        let x = unsafe {
            device_info
                .logical_device
                .create_image(&image_info, None)
                .expect("failed to create image")
        };

        let mem_requirements =
            unsafe { device_info.logical_device.get_image_memory_requirements(x) };

        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device_info._physical_device) };

        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Self::find_memory_type(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
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

    fn find_memory_type(
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> u32 {
        for (i, memory_type) in memory_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(properties) {
                return i as u32;
            }
        }

        panic!()
    }

    fn create_uniform_buffers(
        device_info: &DeviceInfo,
        instance: &ash::Instance,
    ) -> (Vec<BufferInfo>, Vec<*mut UniformBufferObject>) {
        let buffer_size = mem::size_of::<UniformBufferObject>() as u64;

        let mut uniform_buffers = vec![];
        let mut uniform_buffers_mapped = vec![];

        for _frame in 0..constants::MAX_FRAMES_IN_FLIGHT {
            let buffer = BufferInfo::new(
                instance,
                device_info,
                buffer_size,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
            );

            let uniform_buffer_mapped = unsafe {
                device_info
                    .logical_device
                    .map_memory(
                        buffer.buffer_memory,
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("failed to map uniform buffer memory")
                    as *mut UniformBufferObject
            };

            uniform_buffers.push(buffer);
            uniform_buffers_mapped.push(uniform_buffer_mapped);
        }

        (uniform_buffers, uniform_buffers_mapped)
    }

    fn create_descriptor_set_layout(device_info: &DeviceInfo) -> vk::DescriptorSetLayout {
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let sampler_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let x = vec![ubo_layout_binding, sampler_layout_binding];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&x);

        unsafe {
            device_info
                .logical_device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Failed to create descriptor set layout")
        }
    }

    fn create_index_buffer(instance: &ash::Instance, device_info: &DeviceInfo) -> BufferInfo {
        let buffer_size = std::mem::size_of_val(&INDICES) as u64;
        let staging_buffer = buffer::BufferInfo::new(
            instance,
            device_info,
            buffer_size,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::BufferUsageFlags::TRANSFER_SRC,
        );

        unsafe {
            let data = device_info
                .logical_device
                .map_memory(
                    staging_buffer.buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to map memory") as *mut u16;

            data.copy_from_nonoverlapping(INDICES.as_ptr(), INDICES.len());

            device_info
                .logical_device
                .unmap_memory(staging_buffer.buffer_memory);
        };

        let index_buffer = buffer::BufferInfo::new(
            instance,
            device_info,
            buffer_size,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        );

        index_buffer.copy_buffer(&staging_buffer, buffer_size, device_info);

        staging_buffer.destroy_buffer(&device_info.logical_device);

        index_buffer
    }

    fn create_vertex_buffer(instance: &ash::Instance, device_info: &DeviceInfo) -> BufferInfo {
        let buffer_size = std::mem::size_of_val(&VERTICES) as u64;

        let staging_buffer = buffer::BufferInfo::new(
            instance,
            device_info,
            buffer_size,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::BufferUsageFlags::TRANSFER_SRC,
        );

        unsafe {
            let data = device_info
                .logical_device
                .map_memory(
                    staging_buffer.buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to map memory") as *mut Vertex;

            data.copy_from_nonoverlapping(VERTICES.as_ptr(), VERTICES.len());

            device_info
                .logical_device
                .unmap_memory(staging_buffer.buffer_memory);
        };

        let vertex_buffer = buffer::BufferInfo::new(
            instance,
            device_info,
            buffer_size,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        );

        vertex_buffer.copy_buffer(&staging_buffer, buffer_size, device_info);

        staging_buffer.destroy_buffer(&device_info.logical_device);

        vertex_buffer
    }

    fn create_render_pass(
        swapchain_info: &SwapchainInfo,
        device: &ash::Device,
    ) -> ash::vk::RenderPass {
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

    fn create_instance(entry: &ash::Entry, window: &Window) -> ash::Instance {
        let app_name = CString::new("Vulkan Application").unwrap();
        let engine_name = CString::new("No Engine").unwrap();

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name.as_c_str())
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(engine_name.as_c_str())
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap();
        let mut extension_names = extension_names.to_vec();
        if validation::VALIDATION.is_enabled {
            extension_names.push(vk::EXT_DEBUG_UTILS_NAME.as_ptr());
        }

        extension_names.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME.as_ptr());

        let requred_validation_layer_raw_names: Vec<CString> = validation::VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let enable_layer_names: Vec<*const i8> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let p_debug_utils_messenger_info = if validation::VALIDATION.is_enabled {
            &validation::populate_debug_messenger_create_info()
                as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
        } else {
            ptr::null()
        };
        let instance_create_info = vk::InstanceCreateInfo {
            s_type: ash::vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: p_debug_utils_messenger_info,
            pp_enabled_layer_names: enable_layer_names.as_ptr(),
            enabled_layer_count: enable_layer_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            p_application_info: &app_info,
            ..Default::default()
        };

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn create_image_views(
        swapchain_info: &SwapchainInfo,
        device: &ash::Device,
    ) -> Vec<vk::ImageView> {
        let mut image_views = vec![];

        for swapchain_image in swapchain_info.swapchain_images.clone() {
            let image_view_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: swapchain_image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: swapchain_info.swapchain_image_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };

            let image_view_result =
                unsafe { device.create_image_view(&image_view_create_info, None) };

            let image_view = match image_view_result {
                Ok(image_view) => image_view,
                _ => panic!("Error creating image view"),
            };

            image_views.push(image_view);
        }

        image_views
    }

    fn recreate_swapchain(&mut self) {
        unsafe { self.device_info.logical_device.device_wait_idle().unwrap() }

        self.cleanup_swapchain();

        self.device_info
            .update_swapchain_capabilities(&self.surface_info);
        self.swapchain_info =
            swapchain::SwapchainInfo::new(&self.instance, &self.device_info, &self.surface_info);
        self.image_views =
            Self::create_image_views(&self.swapchain_info, &self.device_info.logical_device);
        self.swapchain_frame_buffers = frame_buffer::create_buffers(
            &self.device_info.logical_device,
            &self.image_views,
            &self.render_pass,
            &self.swapchain_info.swapchain_extent,
        );
    }

    fn cleanup_swapchain(&mut self) {
        for framebuffer in self.swapchain_frame_buffers.iter() {
            unsafe {
                self.device_info
                    .logical_device
                    .destroy_framebuffer(*framebuffer, None)
            }
        }

        for image_view in self.image_views.iter() {
            unsafe {
                self.device_info
                    .logical_device
                    .destroy_image_view(*image_view, None)
            }
        }

        unsafe {
            self.swapchain_info
                .swapchain_device
                .destroy_swapchain(self.swapchain_info.swapchain, None)
        }
    }

    fn create_sync_objects(
        device: &ash::Device,
    ) -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>) {
        let semaphore_create_info = ash::vk::SemaphoreCreateInfo {
            s_type: ash::vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_create_info = ash::vk::FenceCreateInfo {
            s_type: ash::vk::StructureType::FENCE_CREATE_INFO,
            flags: ash::vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut in_flight_fences = vec![];

        for _frame in 0..constants::MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("");

                image_available_semaphores.push(image_available_semaphore);

                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("");
                render_finished_semaphores.push(render_finished_semaphore);

                let in_flight_fence = device.create_fence(&fence_create_info, None).expect("");
                in_flight_fences.push(in_flight_fence);
            }
        }

        (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
        )
    }

    fn update_uniform_buffer(&mut self, current_frame: u32, delta_time: f32) {
        self.ubo.model =
            Matrix4::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), cgmath::Deg(90.0) * delta_time)
                * self.ubo.model;

        let current_mapped_memory = self.uniform_buffers_mapped[current_frame as usize];

        let slice = slice::from_ref(&self.ubo);

        unsafe { current_mapped_memory.copy_from_nonoverlapping(slice.as_ptr(), slice.len()) };
    }

    pub fn begin_render_pass(&self, image_index: u32, current_frame: u32) {
        let clear_values = [ash::vk::ClearValue {
            color: ash::vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_begin_info = ash::vk::RenderPassBeginInfo {
            s_type: ash::vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass,
            framebuffer: self.swapchain_frame_buffers[image_index as usize],
            render_area: ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_info.swapchain_extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        unsafe {
            self.device_info.logical_device.cmd_begin_render_pass(
                self.command_buffer_info.command_buffers[current_frame as usize],
                &render_pass_begin_info,
                ash::vk::SubpassContents::INLINE,
            );

            self.device_info.logical_device.cmd_bind_pipeline(
                self.command_buffer_info.command_buffers[current_frame as usize],
                ash::vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_info.graphics_pipelines[0],
            );

            let viewport = ash::vk::Viewport {
                x: 0.0f32,
                y: 0.0f32,
                width: self.swapchain_info.swapchain_extent.width as f32,
                height: self.swapchain_info.swapchain_extent.height as f32,
                min_depth: 0 as f32,
                max_depth: 1f32,
            };

            self.device_info.logical_device.cmd_set_viewport(
                self.command_buffer_info.command_buffers[current_frame as usize],
                0,
                &[viewport],
            );

            let scissor = ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_info.swapchain_extent,
            };

            self.device_info.logical_device.cmd_set_scissor(
                self.command_buffer_info.command_buffers[current_frame as usize],
                0,
                &[scissor],
            );

            let vertex_buffers = [self.vertex_buffer_info.buffer];
            self.device_info.logical_device.cmd_bind_vertex_buffers(
                self.command_buffer_info.command_buffers[current_frame as usize],
                0,
                &vertex_buffers,
                &[0_u64],
            );

            self.device_info.logical_device.cmd_bind_index_buffer(
                self.command_buffer_info.command_buffers[current_frame as usize],
                self.index_buffer_info.buffer,
                0,
                vk::IndexType::UINT16,
            );

            self.device_info.logical_device.cmd_bind_descriptor_sets(
                self.command_buffer_info.command_buffers[current_frame as usize],
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_info.pipeline_layout,
                0,
                &[self.descriptor_sets[current_frame as usize]],
                &[],
            );

            self.device_info.logical_device.cmd_draw_indexed(
                self.command_buffer_info.command_buffers[current_frame as usize],
                INDICES.len() as u32,
                1,
                0,
                0,
                0,
            );

            self.device_info.logical_device.cmd_end_render_pass(
                self.command_buffer_info.command_buffers[current_frame as usize],
            );

            self.device_info
                .logical_device
                .end_command_buffer(
                    self.command_buffer_info.command_buffers[current_frame as usize],
                )
                .expect("Failed to end command buffer");
        }
    }

    pub fn draw_frame(&mut self, delta_time: f32) {
        let current_in_flight_fence = self.in_flight_fences[self.current_frame as usize];
        let current_image_available_semaphore =
            self.image_available_semaphores[self.current_frame as usize];
        let current_render_finished_semaphore =
            self.render_finished_semaphores[self.current_frame as usize];
        let current_command_buffer =
            self.command_buffer_info.command_buffers[self.current_frame as usize];

        unsafe {
            self.device_info
                .logical_device
                .wait_for_fences(&[current_in_flight_fence], true, u64::MAX)
                .expect("Unable to wait for fence")
        }

        let image_result = unsafe {
            self.swapchain_info.swapchain_device.acquire_next_image(
                self.swapchain_info.swapchain,
                u64::MAX,
                current_image_available_semaphore,
                vk::Fence::null(),
            )
        };

        let (image_index, _is_sub_optimal) = match image_result {
            Ok(result) => result,
            Err(error_result) => match error_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR => {
                    self.recreate_swapchain();
                    println!("Error SWAPCHAIN");
                    return;
                }
                _ => panic!(),
            },
        };

        unsafe {
            self.device_info
                .logical_device
                .reset_fences(&[current_in_flight_fence])
                .expect("Unable to reset fence")
        };

        unsafe {
            self.device_info
                .logical_device
                .reset_command_buffer(current_command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("Unable to reset command buffer");
        }

        self.command_buffer_info
            .record_command_buffer(&self.device_info.logical_device, self.current_frame);

        self.begin_render_pass(image_index, self.current_frame);

        self.update_uniform_buffer(self.current_frame, delta_time);

        let semaphore = [current_render_finished_semaphore];

        let binding = [current_image_available_semaphore];
        let binding2 = [self.command_buffer_info.command_buffers[self.current_frame as usize]];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&binding)
            .wait_dst_stage_mask(&[ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&binding2)
            .signal_semaphores(&semaphore);

        unsafe {
            self.device_info
                .logical_device
                .queue_submit(
                    self.device_info.queue_info.graphics_queue,
                    &[submit_info],
                    self.in_flight_fences[self.current_frame as usize],
                )
                .expect("Unable to submit draw command buffer");
        }

        let present_info = ash::vk::PresentInfoKHR {
            s_type: ash::vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: semaphore.as_ptr(),
            p_swapchains: &self.swapchain_info.swapchain,
            swapchain_count: 1,
            p_image_indices: &image_index,
            ..Default::default()
        };

        let present_result = unsafe {
            self.swapchain_info
                .swapchain_device
                .queue_present(self.device_info.queue_info.present_queue, &present_info)
        };

        match present_result {
            Ok(result) => result,
            Err(error_result) => match error_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR => {
                    self.recreate_swapchain();
                    println!("Error SWAPCHAIN");
                    return;
                }
                _ => panic!(),
            },
        };

        self.current_frame = (self.current_frame + 1) % constants::MAX_FRAMES_IN_FLIGHT;
    }
}
