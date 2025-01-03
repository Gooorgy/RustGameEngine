use super::{
    buffer::BufferInfo,
    constants,
    device::{self, DeviceInfo},
    image_util,
    structs::{UniformBufferObject, Vertex},
    surface::SurfaceInfo,
    swapchain::{self, SwapchainInfo},
};
use crate::vulkan_render::camera::Camera;
use crate::vulkan_render::constants::MAX_FRAMES_IN_FLIGHT;
use crate::vulkan_render::graphics_pipeline::PipelineInfo;
use crate::vulkan_render::scene::{Mesh, SceneNode};
use crate::vulkan_render::structs::{AllocatedBuffer, AllocatedImage, FrameData, GPUMeshData, UboDynamicData};
use ash::vk::{self, Extent2D, Extent3D, ImageView, Rect2D};
use ash::vk::{DescriptorPool, ImageAspectFlags, MemoryPropertyFlags};
use ash::Instance;
use core::slice;
use nalgebra::Vector3;
use std::cell::RefCell;
use std::rc::Rc;
use std::{error::Error, ffi::CString, mem, ptr};
use winit::{raw_window_handle::HasDisplayHandle, window::Window};

pub struct VulkanBackend {
    _entry: ash::Entry,
    instance: ash::Instance,
    device_info: DeviceInfo,
    surface_info: SurfaceInfo,
    swapchain_info: SwapchainInfo,
    frame_data: Vec<FrameData>,
    frame_index: usize,
    depth_image: AllocatedImage,
    draw_image: AllocatedImage,
    image_views: Vec<ImageView>,
    pipeline: PipelineInfo,
    descriptor_set_layout: vk::DescriptorSetLayout,
    gpu_mesh_data: Vec<GPUMeshData>,
    descriptor_sets: Vec<vk::DescriptorSet>,
    ubo: UniformBufferObject,
    pub uniform_buffers: Vec<AllocatedBuffer>,
    pub dynamic_uniform_buffers: Vec<AllocatedBuffer>,
    pub camera: Camera,
    dynamic_offset: u64,
}

impl VulkanBackend {
    pub fn new(window: &Window, scene: Rc<RefCell<SceneNode>>, terrain_mesh: Mesh) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { ash::Entry::load()? };
        let instance = Self::create_instance(&entry, window);
        let surface_info = SurfaceInfo::new(&entry, &instance, window);
        let device_info = DeviceInfo::new(&instance, &surface_info);

        let swapchain_info = SwapchainInfo::new(&instance, &device_info, &surface_info);
        let frame_data = Self::construct_frame_data(&device_info);

        let draw_image =
            Self::create_draw_image(&instance, &device_info, swapchain_info.swapchain_extent);
        let depth_image = Self::create_depth_resources(&instance, &device_info, &swapchain_info);
        let image_views = Self::create_image_views(&swapchain_info, &device_info);

        let descriptor_set_layout = Self::create_descriptor_set_layout(&device_info);

        let pipeline = PipelineInfo::new(&device_info.logical_device, &descriptor_set_layout);

        let texture_image = Self::create_texture_image(&device_info, &instance);
        let texture_image_view = Self::create_texture_image_view(&device_info, texture_image.0);
        let texture_sampler = Self::create_texture_sampler(&device_info, &instance);

        let descriptor_pool = Self::create_descriptor_pool(
            &device_info,
            swapchain_info.swapchain_images.len() as u32,
        );

        let (uniform_buffers, dynamic_uniform_buffers, dynamic_aligment) =
            Self::create_uniform_buffers(&device_info, &instance, 2);

        let descriptor_sets = Self::create_descriptor_sets(
            &device_info,
            swapchain_info.swapchain_images.len() as u32,
            descriptor_set_layout,
            descriptor_pool,
            &uniform_buffers,
            &dynamic_uniform_buffers,
            dynamic_aligment,
            texture_image_view,
            texture_sampler,
        );

        let aspect_ratio = swapchain_info.swapchain_extent.width as f32
            / swapchain_info.swapchain_extent.height as f32;
        let mut ubo = UniformBufferObject {
            view: glm::look_at(&Vector3::new(2.0,2.0,2.0), &Vector3::new(0.0,0.0,0.0), &Vector3::new(0.0,0.0,1.0)),
            proj: glm::perspective(45.0_f32.to_radians(), aspect_ratio,0.1, 10.0),

        };

        ubo.proj[(1,1)] *= -1.0;

        let gpu_mesh_data = Self::upload_meshes(&instance, &device_info, scene, terrain_mesh);
        Ok(Self {
            _entry: entry,
            instance,
            device_info,
            surface_info,
            swapchain_info,
            frame_data,
            frame_index: 0,
            draw_image,
            depth_image,
            image_views,
            descriptor_set_layout,
            pipeline,
            gpu_mesh_data,
            descriptor_sets,
            ubo,
            uniform_buffers,
            dynamic_uniform_buffers,
            camera: Camera::new(),
            dynamic_offset: dynamic_aligment,
        })
    }

    fn upload_meshes(
        instance: &Instance,
        device_info: &DeviceInfo,
        scene: Rc<RefCell<SceneNode>>,
        mesh: Mesh
    ) -> Vec<GPUMeshData> {
        let node = scene.borrow();
        let vertices = mesh.vertices;
        let indices = mesh.indices;

        let vertex_buffer = Self::create_vertex_buffer(instance, device_info, &vertices);
        let index_buffer = Self::create_index_buffer(instance, device_info, &indices);

        let mut mesh_data = vec![];

        mesh_data.push(GPUMeshData {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            world_model: node.transform.model,
        });

/*        if node.children.len() > 0 {
            let child = node.children[0].clone();
            let mut child_data = Self::upload_meshes(instance, device_info, child, mesh.clone());
            mesh_data.append(&mut child_data);
        }*/

        mesh_data
    }

    fn create_draw_image(
        instance: &Instance,
        device_info: &DeviceInfo,
        extent: Extent2D,
    ) -> AllocatedImage {
        let (image, image_memory) = Self::create_image(
            device_info,
            instance,
            extent.width,
            extent.height,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::STORAGE
                | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let image_view = Self::create_image_view(
            device_info,
            image,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ImageAspectFlags::COLOR,
        );

        AllocatedImage {
            image,
            image_memory,
            image_view,
            image_extent: Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            image_format: vk::Format::R16G16B16A16_SFLOAT,
        }
    }

    fn construct_frame_data(device_info: &DeviceInfo) -> Vec<FrameData> {
        let command_buffers = Self::create_command_buffers(device_info);

        let mut frame_data = vec![];

        for _frame in 0..MAX_FRAMES_IN_FLIGHT {
            let (swapchain_semaphore, render_semaphore, render_fence) =
                Self::create_sync_objects(&device_info.logical_device);

            frame_data.push(FrameData {
                swapchain_semaphore,
                render_semaphore,
                render_fence,
                command_buffer: command_buffers[_frame as usize],
            })
        }

        frame_data
    }

    fn get_current_frame(&mut self) -> &FrameData {
        let current_frame = &self.frame_data[self.frame_index];

        let new_frame_index = (self.frame_index as u32 + 1) % (constants::MAX_FRAMES_IN_FLIGHT);
        self.frame_index = new_frame_index as usize;

        current_frame
    }

    pub fn draw_frame(&mut self, delta_time: f32) {
        let current_frame = self.get_current_frame();

        let render_fence = current_frame.render_fence;
        let swapchain_semaphore = current_frame.swapchain_semaphore;
        let render_semaphore = current_frame.render_semaphore;
        let command_buffer = current_frame.command_buffer;

        unsafe {
            self.device_info
                .logical_device
                .wait_for_fences(&[render_fence], true, u64::MAX)
                .expect("Unable to wait for fence")
        }

        let image_result = unsafe {
            self.swapchain_info.swapchain_device.acquire_next_image(
                self.swapchain_info.swapchain,
                u64::MAX,
                swapchain_semaphore,
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
                .reset_fences(&[render_fence])
                .expect("Unable to reset fence")
        };

        unsafe {
            self.device_info
                .logical_device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("Unable to reset command buffer");

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default();

            self.device_info
                .logical_device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("failed to begin command buffer")
        }

        image_util::transition_image_layout(
            &self.device_info,
            &command_buffer,
            self.draw_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
            false,
        );

        let color_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.draw_image.image_view)
            .image_layout(vk::ImageLayout::GENERAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE);

        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.depth_image.image_view)
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            });

        let color_attachments = [color_attachment];
        let begin_render_info = vk::RenderingInfo::default()
            .render_area(Rect2D {
                extent: self.swapchain_info.swapchain_extent,
                offset: vk::Offset2D { x: 0, y: 0 },
            })
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&depth_attachment);

        unsafe {
            self.device_info
                .logical_device
                .cmd_begin_rendering(command_buffer, &begin_render_info)
        }

        let viewport = ash::vk::Viewport {
            x: 0.0f32,
            y: 0.0f32,
            width: self.swapchain_info.swapchain_extent.width as f32,
            height: self.swapchain_info.swapchain_extent.height as f32,
            min_depth: 0 as f32,
            max_depth: 1f32,
        };

        unsafe {
            self.device_info
                .logical_device
                .cmd_set_viewport(command_buffer, 0, &[viewport]);
        }

        let scissor = ash::vk::Rect2D {
            offset: ash::vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain_info.swapchain_extent,
        };

        unsafe {
            self.device_info
                .logical_device
                .cmd_set_scissor(command_buffer, 0, &[scissor]);
        }

        unsafe {
            self.device_info.logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.graphics_pipelines[0],
            );
        }
        let aspect_ratio = self.swapchain_info.swapchain_extent.width as f32
            / self.swapchain_info.swapchain_extent.height as f32;

        let view = self.camera.get_view_matrix();
        let mut projection = glm::perspective(aspect_ratio,70_f32.to_radians(), 0.01, 10000.0);
        projection[(1,1)] *= -1.0;

        self.ubo.view = view;
        self.ubo.proj = projection;

        self.update_dynamic_uniform_buffers(self.frame_index);

        for (i,gpu_mesh) in self.gpu_mesh_data.iter_mut().enumerate() {

            let x = self.descriptor_sets[self.frame_index];

            unsafe {
                self.device_info.logical_device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline.pipeline_layout,
                    0,
                    &[x],
                    &[(i as u32 * self.dynamic_offset as u32) ],
                )
            }

            let frame_index = self.frame_index;
            let current_mapped_memory = self.uniform_buffers[frame_index].mapped_buffer as *mut UniformBufferObject;

            let slice = slice::from_ref(&self.ubo);

            unsafe { current_mapped_memory.copy_from_nonoverlapping(slice.as_ptr(), slice.len()) };

            unsafe {
                self.device_info.logical_device.cmd_bind_index_buffer(
                    command_buffer,
                    gpu_mesh.index_buffer.buffer,
                    0,
                    vk::IndexType::UINT32,
                );

                self.device_info.logical_device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &[gpu_mesh.vertex_buffer.buffer],
                    &[0],
                );

                self.device_info.logical_device.cmd_draw_indexed(
                    command_buffer,
                    gpu_mesh.index_count,
                    1,
                    0,
                    0,
                    0,
                );
            }
        }

        unsafe {
            self.device_info
                .logical_device
                .cmd_end_rendering(command_buffer);

            image_util::transition_image_layout(
                &self.device_info,
                &command_buffer,
                self.draw_image.image,
                vk::ImageLayout::GENERAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                false,
            );
            image_util::transition_image_layout(
                &self.device_info,
                &command_buffer,
                self.swapchain_info.swapchain_images[image_index as usize],
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                false,
            );

            let draw_extend = Extent2D {
                height: self.draw_image.image_extent.height,
                width: self.draw_image.image_extent.width,
            };

            image_util::copy_image_to_image(
                &self.device_info.logical_device,
                &command_buffer,
                self.draw_image.image,
                self.swapchain_info.swapchain_images[image_index as usize],
                draw_extend,
                self.swapchain_info.swapchain_extent,
            );
            image_util::transition_image_layout(
                &self.device_info,
                &command_buffer,
                self.swapchain_info.swapchain_images[image_index as usize],
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                false,
            );
            image_util::transition_image_layout(
                &self.device_info,
                &command_buffer,
                self.swapchain_info.swapchain_images[image_index as usize],
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::PRESENT_SRC_KHR,
                false,
            );

            self.device_info
                .logical_device
                .end_command_buffer(command_buffer)
                .expect("failed to end command buffer");
        }

        let command_buffer_submit_info = [vk::CommandBufferSubmitInfo::default()
            .command_buffer(command_buffer)
            .device_mask(0)];

        let wait_info = [vk::SemaphoreSubmitInfo::default()
            .semaphore(swapchain_semaphore)
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT_KHR)
            .device_index(0)
            .value(1)];

        let signal_info = [vk::SemaphoreSubmitInfo::default()
            .semaphore(render_semaphore)
            .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
            .device_index(0)
            .value(1)];

        let submit_info = vk::SubmitInfo2::default()
            .command_buffer_infos(&command_buffer_submit_info)
            .wait_semaphore_infos(&wait_info)
            .signal_semaphore_infos(&signal_info);

        unsafe {
            self.device_info
                .logical_device
                .queue_submit2(
                    self.device_info.queue_info.graphics_queue,
                    &[submit_info],
                    render_fence,
                )
                .expect("Unable to submit draw command buffer");
        }

        let render_semaphores = [render_semaphore];
        let swapchains = [self.swapchain_info.swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&render_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

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
    }

    fn create_sync_objects(device: &ash::Device) -> (vk::Semaphore, vk::Semaphore, vk::Fence) {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        unsafe {
            let swapchain_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("");

            let render_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("");

            let render_fence = device.create_fence(&fence_create_info, None).expect("");

            (swapchain_semaphore, render_semaphore, render_fence)
        }
    }

    fn create_depth_resources(
        instance: &ash::Instance,
        device_info: &DeviceInfo,
        swapchain_info: &SwapchainInfo,
    ) -> AllocatedImage {
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

        let image_extent = vk::Extent3D {
            width: swapchain_info.swapchain_extent.width,
            height: swapchain_info.swapchain_extent.height,
            depth: 1,
        };

        AllocatedImage {
            image,
            image_view: depth_image_view,
            image_extent,
            image_memory,
            image_format: depth_format,
        }
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
        uniform_buffers: &Vec<AllocatedBuffer>,
        dynamic_buffers: &Vec<AllocatedBuffer>,
        dynamic_alignment: u64,
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
                .buffer(uniform_buffers[i as usize].buffer)
                .offset(0)
                .range(std::mem::size_of::<UniformBufferObject>() as u64);

            let dynamic_buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(dynamic_buffers[i as usize].buffer)
                .offset(0)
                .range(dynamic_alignment);

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

            write_descriptor_sets.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i as usize])
                    .dst_binding(2)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                    .descriptor_count(1)
                    .buffer_info(slice::from_ref(&dynamic_buffer_info)),
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

        pool_sizes.push(
            vk::DescriptorPoolSize::default()
                .descriptor_count(descriptor_count)
                .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC),
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

    fn create_texture_image(
        device_info: &DeviceInfo,
        instance: &ash::Instance,
    ) -> (vk::Image, vk::DeviceMemory) {
        let dyn_image = image::open(".\\resources\\textures\\texture.png").unwrap();
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

        let (image, device_memory) = Self::create_image(
            device_info,
            instance,
            dyn_image.width(),
            dyn_image.height(),
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let cmd = BufferInfo::begin_single_time_command(device_info);

        image_util::transition_image_layout(
            device_info,
            &cmd,
            image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            false,
        );

        BufferInfo::end_single_time_command(device_info, cmd);

        Self::copy_buffy_to_image(
            device_info,
            image_buffer.buffer,
            image,
            image_width,
            image_height,
        );
        let cmd = BufferInfo::begin_single_time_command(device_info);

        image_util::transition_image_layout(
            device_info,
            &cmd,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            false,
        );

        BufferInfo::end_single_time_command(device_info, cmd);

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
        mesh_count: u32,
    ) -> (
        Vec<AllocatedBuffer>,
        Vec<AllocatedBuffer>,
        u64
    ) {
        let mut dynamic_alignment = mem::size_of::<UboDynamicData>() as u64;
        let min_ubo_alignment = device_info.min_ubo_alignment;

        if(min_ubo_alignment > 0) {
            dynamic_alignment = (dynamic_alignment + min_ubo_alignment - 1) & !(min_ubo_alignment - 1);
        }

        let buffer_size = mem::size_of::<UniformBufferObject>() as u64;
        let dynamic_buffer_size = dynamic_alignment * mesh_count as u64;

        let mut uniform_buffers = vec![];
        let mut dynamic_uniform_buffers = vec![];

        for _frame in 0..constants::MAX_FRAMES_IN_FLIGHT {
            let (buffer, device_memory) = Self::create_buffer(
                instance,
                device_info,
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );

            let mapped_memory = unsafe {
                device_info
                    .logical_device
                    .map_memory(device_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                    .expect("failed to map memory")
            };

            uniform_buffers.push(
                AllocatedBuffer {
                    buffer,
                    buffer_memory: device_memory,
                    mapped_buffer: mapped_memory,
                }
            );

            let (buffer, device_memory) = Self::create_buffer(
                instance,
                device_info,
                dynamic_buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE,
            );

            let mapped_memory = unsafe {
                device_info
                    .logical_device
                    .map_memory(device_memory, 0, dynamic_buffer_size, vk::MemoryMapFlags::empty())
                    .expect("failed to map memory")
            };

            dynamic_uniform_buffers.push(
                AllocatedBuffer {
                    buffer,
                    buffer_memory: device_memory,
                    mapped_buffer: mapped_memory,
                }
            )


        }

        (
            uniform_buffers,
            dynamic_uniform_buffers,
            dynamic_alignment
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

        let dynamic_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let x = vec![ubo_layout_binding, sampler_layout_binding, dynamic_layout_binding];

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
    ) -> AllocatedBuffer {
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

        AllocatedBuffer {
            buffer: index_buffer,
            buffer_memory: index_buffer_memory,
            mapped_buffer: ptr::null_mut(),
        }
    }

    fn create_vertex_buffer(
        instance: &ash::Instance,
        device_info: &DeviceInfo,
        vertices: &[Vertex],
    ) -> AllocatedBuffer {
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

        let (vertex_buffer, vertex_buffer_memory) = Self::create_buffer(
            instance,
            device_info,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        Self::copy_buffer(x, vertex_buffer, buffer_size, device_info);

        unsafe {
            device_info.logical_device.destroy_buffer(x, None);
            device_info.logical_device.free_memory(y, None);
        }

        AllocatedBuffer {
            buffer: vertex_buffer,
            buffer_memory: vertex_buffer_memory,
            mapped_buffer: ptr::null_mut(),
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
            .api_version(vk::make_api_version(0, 1, 3, 0));

        let extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap();
        let mut extension_names = extension_names.to_vec();

        extension_names.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME.as_ptr());

        let instance_create_info = vk::InstanceCreateInfo {
            s_type: ash::vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
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
        let depth_image =
            Self::create_depth_resources(&self.instance, &self.device_info, &self.swapchain_info);

        self.depth_image = depth_image;
    }

    fn cleanup_swapchain(&mut self) {
        for framebuffer in self.swapchain_info.frame_buffers.iter() {
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

    fn update_dynamic_uniform_buffers(&mut self, image_index: usize) {

        let x = self.gpu_mesh_data.iter().map(|data| UboDynamicData {
            model: data.world_model,
        }).collect::<Vec<_>>();

/*        for gpu_mesh in self.gpu_mesh_data.iter() {
            let x = gpu_mesh.world_model;
            let data = UboDynamicData {
                model: x,
            };
            let slice = slice::from_ref(&data);

        }*/

        let mem = self.dynamic_uniform_buffers[image_index].mapped_buffer as *mut UboDynamicData;

            unsafe { mem.copy_from_nonoverlapping(x.as_ptr(),x.len()) };
        let x = vk::MappedMemoryRange::default().memory(self.dynamic_uniform_buffers[image_index].buffer_memory).size(
            self.dynamic_offset
        );

        unsafe { self.device_info.logical_device.flush_mapped_memory_ranges(&[x]).expect("d"); }
    }
}
