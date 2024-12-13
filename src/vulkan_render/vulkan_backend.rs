use ash::vk::{self};
use ash::vk::{CommandBuffer, DescriptorPool, ImageAspectFlags, MemoryPropertyFlags};
use ash::Device;
use cgmath::{assert_abs_diff_eq, Matrix4, Point3, Vector2, Vector3};
use core::slice;
use image::{DynamicImage, GenericImageView, ImageReader};
use std::fmt::Debug;
use std::io::BufReader;
use std::time::Instant;
use std::{
    error::Error,
    ffi::{c_void, CString},
    fs, mem, ptr,
};
use winit::{raw_window_handle::HasDisplayHandle, window::Window};
use crate::{INDICES, VERTICES};
use super::{
    buffer::{self, BufferInfo},
    constants, descriptors,
    device::{self, DeviceInfo},
    frame_buffer,
    graphics_pipeline::PipelineInfo,
    structs::{UniformBufferObject, Vertex},
    surface::{self, SurfaceInfo},
    swapchain::{self, SwapchainInfo},
    validation,
};
use crate::vulkan_render::constants::MAX_FRAMES_IN_FLIGHT;

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
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<ash::vk::Fence>,
    command_buffer_info: Vec<vk::CommandBuffer>,
    current_frame: u32,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,

    descriptor_set_layout: vk::DescriptorSetLayout,

    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffer_memory: Vec<vk::DeviceMemory>,
    uniform_buffers_mapped: Vec<*mut UniformBufferObject>,

    descriptor_sets: Vec<vk::DescriptorSet>,
    start_time: Instant,
    ubo: UniformBufferObject,
    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl VulkanBackend {
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { ash::Entry::load()? };
        let instance = Self::create_instance(&entry, window);
        let surface_info = surface::SurfaceInfo::new(&entry, &instance, window);
        let device_info = device::DeviceInfo::new(&instance, &surface_info);
        let swapchain_info = swapchain::SwapchainInfo::new(&instance, &device_info, &surface_info);

        let image_views = Self::create_image_views(&swapchain_info, &device_info);

        let render_pass = Self::create_render_pass(&device_info, &instance, &swapchain_info);
        let descriptor_set_layout = Self::create_descriptor_set_layout(&device_info);
        let pipeline_info = PipelineInfo::new(
            &render_pass,
            &device_info.logical_device,
            &descriptor_set_layout,
        );

        let (depth_image, depth_image_memory, depth_image_view) =
            Self::create_depth_resources(&instance, &device_info, &swapchain_info);

        let frame_buffers = frame_buffer::create_buffers(
            &device_info.logical_device,
            &image_views,
            depth_image_view,
            &render_pass,
            &swapchain_info.swapchain_extent,
        );

        let (texture_image, texture_image_memory) =
            Self::create_texture_image(&device_info, &instance);
        let texture_image_view = Self::create_texture_image_view(&device_info, texture_image);
        let texture_sampler = Self::create_texture_sampler(&device_info, &instance);

        let (vertices, indices) = Self::load_model();

        let (vertex_buffer, vertex_buffer_memory) =
            Self::create_vertex_buffer(&instance, &device_info, &vertices);
        let (index_buffer, index_buffer_memory) =
            Self::create_index_buffer(&instance, &device_info, &indices);
        let (uniform_buffers, uniform_buffer_memory, uniform_buffers_mapped) =
            Self::create_uniform_buffers(&device_info, &instance);

        let descriptor_pool = Self::create_descriptor_pool(
            &device_info,
            swapchain_info.swapchain_images.len() as u32,
        );

        let descriptor_sets = Self::create_descriptor_sets(
            &device_info,
            swapchain_info.swapchain_images.len() as u32,
            descriptor_set_layout,
            descriptor_pool,
            &uniform_buffers,
            texture_image_view,
            texture_sampler,
        );

        let command_buffer_info = Self::create_command_buffers(&device_info);

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

            vertex_buffer,
            vertex_buffer_memory,

            index_buffer,
            index_buffer_memory,
            descriptor_set_layout,
            uniform_buffers,
            uniform_buffer_memory,
            uniform_buffers_mapped,
            descriptor_sets,
            start_time,
            ubo,
            texture_image,
            texture_image_memory,
            texture_image_view,
            texture_sampler,
            depth_image_view,
            depth_image,
            depth_image_memory,
            vertices,
            indices,
        })
    }

    fn load_model() -> (Vec<Vertex>, Vec<u32>) {
        let load_options = tobj::LoadOptions {
            single_index: true,
            ..Default::default()
        };

        let (models, mat) = tobj::load_obj("E:\\rust\\new\\src\\models\\test.obj", &tobj::GPU_LOAD_OPTIONS)
            .expect("failed to load model file");

        let model = models.first().unwrap();
        let x = &model.mesh;

        println!("indices len: {}", x.indices.len());
        println!("texind len: {}", x.texcoord_indices.len());

        println!("pos len: {}", x.positions.len());
        println!("texcoord len: {}", x.texcoords.len());

        let mut vertices = vec![];

        let vert_count = x.positions.len() / 3;
        for i in 0..vert_count {
            let pos: Vector3<f32> = Vector3::new(
                x.positions[i * 3],
                x.positions[i * 3 + 1],
                x.positions[i * 3 + 2],
            );

            let tex_coord: Vector2<f32> = Vector2::new(
                x.texcoords[i * 2],
                x.texcoords[i * 2 + 1],
            );

            let vert = Vertex {
                pos,
                color: Vector3::new(1.0, 1.0, 1.0),
                tex_coord,
            };

            vertices.push(vert);
        }

        (vertices, x.indices.clone())
    }

    fn create_depth_resources(
        instance: &ash::Instance,
        device_info: &DeviceInfo,
        swapchain_info: &SwapchainInfo,
    ) -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
        let depth_format = Self::find_depth_format(instance, device_info);

        let (image, image_memory) = Self::create_image(
            device_info,
            instance,
            swapchain_info.swapchain_extent.width,
            swapchain_info.swapchain_extent.height,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let depth_image_view = Self::create_image_view(
            device_info,
            image,
            depth_format,
            vk::ImageAspectFlags::DEPTH,
        );

        (image, image_memory, depth_image_view)
    }

    fn create_image_view(
        device_info: &DeviceInfo,
        image: vk::Image,
        format: vk::Format,
        image_aspect_flags: vk::ImageAspectFlags,
    ) -> vk::ImageView {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
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

    fn find_depth_format(instance: &ash::Instance, device_info: &DeviceInfo) -> vk::Format {
        Self::find_supported_format(
            instance,
            device_info,
            vec![
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    fn has_stencil_component(format: vk::Format) -> bool {
        format == vk::Format::D32_SFLOAT || format == vk::Format::D24_UNORM_S8_UINT
    }

    fn find_supported_format(
        instance: &ash::Instance,
        device_info: &DeviceInfo,
        candidates: Vec<vk::Format>,
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        for format in candidates {
            let format_props = unsafe {
                instance.get_physical_device_format_properties(device_info._physical_device, format)
            };

            if tiling == vk::ImageTiling::LINEAR
                && (format_props.linear_tiling_features & features) == features
            {
                return format;
            } else if tiling == vk::ImageTiling::OPTIMAL
                && (format_props.optimal_tiling_features & features) == features
            {
                return format;
            }
        }

        panic!("Failed to find supported format");
    }

    fn record_command_buffer(&self) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            self.device_info
                .logical_device
                .begin_command_buffer(
                    self.command_buffer_info[self.current_frame as usize],
                    &command_buffer_begin_info,
                )
                .expect("failed to begin command buffer")
        };
    }

    fn create_command_buffers(device_info: &DeviceInfo) -> Vec<vk::CommandBuffer> {
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(device_info.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT);

        unsafe {
            device_info
                .logical_device
                .allocate_command_buffers(&command_buffer_alloc_info)
                .expect("failed to allocate command buffer")
        }
    }

    fn create_descriptor_sets(
        device_info: &DeviceInfo,
        descriptor_count: u32,
        descriptor_set_layout: vk::DescriptorSetLayout,
        descriptor_pool: DescriptorPool,
        uniform_buffers: &Vec<vk::Buffer>,
        texture_image_view: vk::ImageView,
        texture_sampler: vk::Sampler,
    ) -> Vec<vk::DescriptorSet> {
        let mut layouts = vec![];
        for _ in 0..descriptor_count {
            layouts.push(descriptor_set_layout);
        }

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe {
            device_info
                .logical_device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to create descriptor sets")
        };

        for i in 0..descriptor_count {
            let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(uniform_buffers[i as usize])
                .offset(0)
                .range(std::mem::size_of::<UniformBufferObject>() as u64);

            let image_info = [vk::DescriptorImageInfo::default()
                .image_view(texture_image_view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .sampler(texture_sampler)];

            let mut write_descriptor_sets = vec![];

            write_descriptor_sets.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i as usize])
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .buffer_info(slice::from_ref(&buffer_info)),
            );

            write_descriptor_sets.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i as usize])
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(1)
                    .image_info(image_info.as_slice()),
            );
            println!("Write descriptor sets");
            unsafe {
                device_info
                    .logical_device
                    .update_descriptor_sets(&write_descriptor_sets, &[]);
            }
        }

        descriptor_sets
    }

    fn create_descriptor_pool(
        device_info: &DeviceInfo,
        descriptor_count: u32,
    ) -> vk::DescriptorPool {
        let mut pool_sizes: Vec<vk::DescriptorPoolSize> = vec![];

        pool_sizes.push(
            vk::DescriptorPoolSize::default()
                .descriptor_count(descriptor_count)
                .ty(vk::DescriptorType::UNIFORM_BUFFER),
        );

        pool_sizes.push(
            vk::DescriptorPoolSize::default()
                .descriptor_count(descriptor_count)
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER),
        );

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(descriptor_count);

        unsafe {
            device_info
                .logical_device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("failed to create descriptor pool!")
        }
    }

    fn create_texture_sampler(device_info: &DeviceInfo, instance: &ash::Instance) -> vk::Sampler {
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
        Self::create_image_view(
            device_info,
            image,
            vk::Format::R8G8B8A8_SRGB,
            ImageAspectFlags::COLOR,
        )
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
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
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
    ) -> (vk::Image, vk::DeviceMemory) {
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

        let (mut image, device_memory) = Self::create_image(
            device_info,
            instance,
            dyn_image.width(),
            dyn_image.height(),
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

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
            device_info
                .logical_device
                .destroy_buffer(image_buffer.buffer, None);
            device_info
                .logical_device
                .free_memory(image_buffer.buffer_memory, None);
        }

        (image, device_memory)
    }

    fn create_image(
        device_info: &DeviceInfo,
        instance: &ash::Instance,
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

        let mem_requirements =
            unsafe { device_info.logical_device.get_image_memory_requirements(x) };

        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device_info._physical_device) };

        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Self::find_memory_type(
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

    fn create_buffer(
        instance: &ash::Instance,
        device_info: &DeviceInfo,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        memory_property_flags: MemoryPropertyFlags,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device_info
                .logical_device
                .create_buffer(&buffer_create_info, None)
                .expect("failed to create buffer")
        };

        let mem_requirements = unsafe {
            device_info
                .logical_device
                .get_buffer_memory_requirements(buffer)
        };
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device_info._physical_device) };
        let memory_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Self::find_memory_type(
                mem_requirements.memory_type_bits,
                memory_property_flags,
                memory_properties,
            ));

        let buffer_memory = unsafe {
            device_info
                .logical_device
                .allocate_memory(&memory_alloc_info, None)
                .expect("failed to allocate memory")
        };

        unsafe {
            device_info
                .logical_device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("failed to bind buffer");
        }

        (buffer, buffer_memory)
    }

    fn create_uniform_buffers(
        device_info: &DeviceInfo,
        instance: &ash::Instance,
    ) -> (
        Vec<vk::Buffer>,
        Vec<vk::DeviceMemory>,
        Vec<*mut UniformBufferObject>,
    ) {
        let buffer_size = mem::size_of::<UniformBufferObject>() as u64;

        let mut uniform_buffers = vec![];
        let mut uniform_buffers_memory = vec![];
        let mut uniform_buffers_mapped = vec![];

        for _frame in 0..constants::MAX_FRAMES_IN_FLIGHT {
            let (buffer, device_memory) = Self::create_buffer(
                instance,
                device_info,
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );

            uniform_buffers.push(buffer);
            uniform_buffers_memory.push(device_memory);

            let mapped_memory = unsafe {
                device_info
                    .logical_device
                    .map_memory(device_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                    .expect("failed to map memory") as *mut UniformBufferObject
            };

            uniform_buffers_mapped.push(mapped_memory);
        }

        (
            uniform_buffers,
            uniform_buffers_memory,
            uniform_buffers_mapped,
        )
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

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&x);

        unsafe {
            device_info
                .logical_device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Failed to create descriptor set layout")
        }
    }

    fn copy_buffer(
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: u64,
        device_info: &device::DeviceInfo,
    ) {
        let command_buffer = BufferInfo::begin_single_time_command(&device_info);

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        };

        unsafe {
            device_info.logical_device.cmd_copy_buffer(
                command_buffer,
                src_buffer,
                dst_buffer,
                &[copy_region],
            )
        };

        BufferInfo::end_single_time_command(device_info, command_buffer);
    }

    fn create_index_buffer(
        instance: &ash::Instance,
        device_info: &DeviceInfo,
        indices: &[u32],
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_size = std::mem::size_of_val(indices) as vk::DeviceSize;

        let (x, y) = Self::create_buffer(
            instance,
            device_info,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            let data = device_info
                .logical_device
                .map_memory(y, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to map memory") as *mut u32;

            data.copy_from_nonoverlapping(indices.as_ptr(), indices.len());

            device_info.logical_device.unmap_memory(y);
        };

        let (index_buffer, index_buffer_memory) = Self::create_buffer(
            instance,
            device_info,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        Self::copy_buffer(x, index_buffer, buffer_size, device_info);

        unsafe {
            device_info.logical_device.destroy_buffer(x, None);
            device_info.logical_device.free_memory(y, None);
        }

        (index_buffer, index_buffer_memory)
    }

    fn create_vertex_buffer(
        instance: &ash::Instance,
        device_info: &DeviceInfo,
        vertices: &[Vertex],
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_size = std::mem::size_of_val(vertices) as u64;

        let (x, y) = Self::create_buffer(
            instance,
            device_info,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            let data = device_info
                .logical_device
                .map_memory(y, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to map memory") as *mut Vertex;

            data.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());

            device_info.logical_device.unmap_memory(y);
        };

        let (index_buffer, index_buffer_memory) = Self::create_buffer(
            instance,
            device_info,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        Self::copy_buffer(x, index_buffer, buffer_size, device_info);

        unsafe {
            device_info.logical_device.destroy_buffer(x, None);
            device_info.logical_device.free_memory(y, None);
        }

        (index_buffer, index_buffer_memory)
    }

    fn create_render_pass(
        device_info: &DeviceInfo,
        instance: &ash::Instance,
        swapchain_info: &SwapchainInfo,
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

        let depth_attachment = vk::AttachmentDescription::default()
            .format(Self::find_depth_format(instance, device_info))
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let depth_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass = ash::vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref);

        let subpass_dependency = ash::vk::SubpassDependency {
            src_subpass: ash::vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | ash::vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: ash::vk::AccessFlags::empty(),
            dst_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | ash::vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | ash::vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let binding = [color_attachment, depth_attachment];
        let binding2 = [subpass_dependency];
        let binding3 = [subpass];
        let render_pass_create_info = ash::vk::RenderPassCreateInfo::default()
            .attachments(&binding)
            .dependencies(&binding2)
            .subpasses(&binding3);

        unsafe {
            device_info
                .logical_device
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
            //extension_names.push(vk::EXT_DEBUG_UTILS_NAME.as_ptr());
        }

        extension_names.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME.as_ptr());

        /*        let requred_validation_layer_raw_names: Vec<CString> = validation::VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let enable_layer_names: Vec<*const i8> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();*/

        let p_debug_utils_messenger_info = if validation::VALIDATION.is_enabled {
            //&validation::populate_debug_messenger_create_info()
            //as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void#
            ptr::null()
        } else {
            ptr::null()
        };
        let instance_create_info = vk::InstanceCreateInfo {
            s_type: ash::vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: p_debug_utils_messenger_info,
            pp_enabled_layer_names: ptr::null(),
            enabled_layer_count: 0,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            p_application_info: &app_info,
            ..Default::default()
        };

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn create_image_views(
        swapchain_info: &SwapchainInfo,
        device_info: &DeviceInfo,
    ) -> Vec<vk::ImageView> {
        let mut image_views = vec![];

        for swapchain_image in swapchain_info.swapchain_images.clone() {
            let image_view = Self::create_image_view(
                device_info,
                swapchain_image,
                swapchain_info.swapchain_image_format.format,
                vk::ImageAspectFlags::COLOR,
            );
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
        self.image_views = Self::create_image_views(&self.swapchain_info, &self.device_info);
        let (depth_image, depth_image_memory, depth_image_view) =
            Self::create_depth_resources(&self.instance, &self.device_info, &self.swapchain_info);

        self.depth_image = depth_image;
        self.depth_image_memory = depth_image_memory;
        self.depth_image_view = depth_image_view;

        self.swapchain_frame_buffers = frame_buffer::create_buffers(
            &self.device_info.logical_device,
            &self.image_views,
            self.depth_image_view,
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
        let clear_values = [
            ash::vk::ClearValue {
                color: ash::vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

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
                self.command_buffer_info[current_frame as usize],
                &render_pass_begin_info,
                ash::vk::SubpassContents::INLINE,
            );

            self.device_info.logical_device.cmd_bind_pipeline(
                self.command_buffer_info[current_frame as usize],
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
                self.command_buffer_info[current_frame as usize],
                0,
                &[viewport],
            );

            let scissor = ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_info.swapchain_extent,
            };

            self.device_info.logical_device.cmd_set_scissor(
                self.command_buffer_info[current_frame as usize],
                0,
                &[scissor],
            );

            let vertex_buffers = [self.vertex_buffer];
            self.device_info.logical_device.cmd_bind_vertex_buffers(
                self.command_buffer_info[current_frame as usize],
                0,
                &vertex_buffers,
                &[0_u64],
            );

            self.device_info.logical_device.cmd_bind_index_buffer(
                self.command_buffer_info[current_frame as usize],
                self.index_buffer,
                0,
                vk::IndexType::UINT32,
            );

            self.device_info.logical_device.cmd_bind_descriptor_sets(
                self.command_buffer_info[current_frame as usize],
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_info.pipeline_layout,
                0,
                &[self.descriptor_sets[current_frame as usize]],
                &[],
            );

            self.device_info.logical_device.cmd_draw_indexed(
                self.command_buffer_info[current_frame as usize],
                self.indices.len() as u32,
                1,
                0,
                0,
                0,
            );

            self.device_info
                .logical_device
                .cmd_end_render_pass(self.command_buffer_info[current_frame as usize]);

            self.device_info
                .logical_device
                .end_command_buffer(self.command_buffer_info[current_frame as usize])
                .expect("Failed to end command buffer");
        }
    }

    pub fn draw_frame(&mut self, delta_time: f32) {
        let current_in_flight_fence = self.in_flight_fences[self.current_frame as usize];
        let current_image_available_semaphore =
            self.image_available_semaphores[self.current_frame as usize];
        let current_render_finished_semaphore =
            self.render_finished_semaphores[self.current_frame as usize];
        let current_command_buffer = self.command_buffer_info[self.current_frame as usize];

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

        self.record_command_buffer();

        self.begin_render_pass(image_index, self.current_frame);

        self.update_uniform_buffer(self.current_frame, delta_time);

        let semaphore = [current_render_finished_semaphore];

        let binding = [current_image_available_semaphore];
        let binding2 = [self.command_buffer_info[self.current_frame as usize]];
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
