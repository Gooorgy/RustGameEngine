use super::{
    buffer::BufferInfo, device::DeviceInfo, image_util, structs::CameraMvpUbo,
    surface::SurfaceInfo, swapchain::SwapchainInfo, utils,
};
use crate::assets::asset_manager::{Asset, MeshAsset};
use crate::vulkan_render::buffer::AllocatedBuffer;
use crate::vulkan_render::camera::Camera;
use crate::vulkan_render::constants::MAX_FRAMES_IN_FLIGHT;
use crate::vulkan_render::frame_manager::FrameManager;
use crate::vulkan_render::image_util::AllocatedImage;
use crate::vulkan_render::render_objects::draw_objects::{Vertex};
use crate::vulkan_render::structs::{
    Cascade, CascadeShadowPushConsts, CascadeShadowUbo, GPUMeshData, LightingUbo, ModelDynamicUbo,
};
use ash::vk::{self, Extent2D, Extent3D, ImageView, PipelineBindPoint, Rect2D, ShaderStageFlags};
use ash::vk::{ImageAspectFlags, MemoryPropertyFlags};
use ash::Instance;
use glm::{vec2, vec3, vec3_to_vec4, vec4, Mat4, Vec3};
use std::rc::Rc;
use std::{error::Error, ffi::CString, mem, ptr};
use winit::{raw_window_handle::HasDisplayHandle, window::Window};

const SHADOW_MAP_CASCADE_COUNT: usize = 3;

pub struct VulkanBackend {
    _entry: ash::Entry,
    instance: Instance,
    device_info: DeviceInfo,
    surface_info: SurfaceInfo,
    swapchain_info: SwapchainInfo,
    image_views: Vec<ImageView>,
    pub camera: Camera,
    frame_manager: FrameManager,
    cascades: Vec<Cascade>,
    cam_view_proj: Mat4,
    gpu_mesh_data: Vec<GPUMeshData>,
}

impl VulkanBackend {
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { ash::Entry::load()? };
        let instance = Self::create_instance(&entry, window);
        let surface_info = SurfaceInfo::new(&entry, &instance, window);
        let device_info = DeviceInfo::new(&instance, &surface_info);
        let swapchain_info = SwapchainInfo::new(&instance, &device_info, &surface_info);

        let image_views = Self::create_image_views(&swapchain_info, &device_info);

        let texture_image = Self::create_texture_image(&device_info, &instance);
        let texture_image_view = Self::create_texture_image_view(&device_info, texture_image.0);
        let texture_sampler = utils::create_texture_sampler(&device_info, &instance, false);

        let frame_manager = FrameManager::new(
            &device_info,
            &instance,
            MAX_FRAMES_IN_FLIGHT as usize,
            swapchain_info.swapchain_extent,
            &texture_sampler,
            &texture_image_view,
            SHADOW_MAP_CASCADE_COUNT,
        );

        let cascades = vec![
            Cascade::default(),
            Cascade::default(),
            Cascade::default(),
            Cascade::default(),
        ];

        Ok(Self {
            _entry: entry,
            instance,
            device_info,
            surface_info,
            swapchain_info,
            image_views,
            camera: Camera::new(),
            frame_manager,
            cascades,
            cam_view_proj: Mat4::identity(),
            gpu_mesh_data: Vec::new(),
        })
    }

    pub fn upload_meshes(
        &mut self,
        meshes: Vec<Rc<Asset<MeshAsset>>>,
        // Find a better way to pass the model information ...
        model: Mat4,
    ) {
        let mut mesh_data = vec![];
        let mesh_count = meshes.len();
        for mesh in meshes {
            let s = &mesh.data.mesh;
            let vertices = &s.vertices;
            let indices = &s.indices;

            let vertex_buffer =
                Self::create_vertex_buffer(&self.instance, &self.device_info, vertices);
            let index_buffer =
                Self::create_index_buffer(&self.instance, &self.device_info, indices);
            println!("Created vertex buffer: vert_count: {}", vertices.len());

            mesh_data.push(GPUMeshData {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
                world_model: model,
            });
        }

        self.gpu_mesh_data = mesh_data;

        self.frame_manager.recreate_model_dynamic_buffer(
            &self.device_info,
            &self.instance,
            mesh_count,
        );
    }

    pub fn draw_frame(&mut self, _delta_time: f32) {
        self.update_camera();
        self.update_world();
        self.update_cascades();
        self.update_cascade_memory();
        self.update_lighting_memory();

        let current_frame = self.frame_manager.get_current_frame();
        unsafe {
            self.device_info
                .logical_device
                .wait_for_fences(&[current_frame.render_fence], true, u64::MAX)
                .expect("Unable to wait for fence")
        }

        let image_result = unsafe {
            self.swapchain_info.swapchain_device.acquire_next_image(
                self.swapchain_info.swapchain,
                u64::MAX,
                current_frame.swapchain_semaphore,
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
                .reset_fences(&[current_frame.render_fence])
                .expect("Unable to reset fence")
        };

        unsafe {
            self.device_info
                .logical_device
                .reset_command_buffer(
                    current_frame.command_buffer,
                    vk::CommandBufferResetFlags::empty(),
                )
                .expect("Unable to reset command buffer");

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default();

            self.device_info
                .logical_device
                .begin_command_buffer(current_frame.command_buffer, &command_buffer_begin_info)
                .expect("failed to begin command buffer")
        }

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.albedo_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
            false,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.pos_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
            false,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.normal_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
            false,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.depth_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            true,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.shadow_cascades[0].shadow_depth_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            true,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.shadow_cascades[1].shadow_depth_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            true,
        );
        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.shadow_cascades[2].shadow_depth_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            true,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.draw_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
            false,
        );

        self.render_cascade_shadow_map();
        self.render_scene();

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.albedo_image.image,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            false,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.pos_image.image,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            false,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.normal_image.image,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            false,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.depth_image.image,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            true,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.shadow_cascades[0].shadow_depth_image.image,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            true,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.shadow_cascades[1].shadow_depth_image.image,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            true,
        );

        image_util::transition_image_layout(
            &self.device_info,
            &current_frame.command_buffer,
            current_frame.shadow_cascades[2].shadow_depth_image.image,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            true,
        );

        self.render_lighting();

        unsafe {
            image_util::transition_image_layout(
                &self.device_info,
                &current_frame.command_buffer,
                current_frame.draw_image.image,
                vk::ImageLayout::GENERAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                false,
            );
            image_util::transition_image_layout(
                &self.device_info,
                &current_frame.command_buffer,
                self.swapchain_info.swapchain_images[image_index as usize],
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                false,
            );

            let draw_extend = Extent2D {
                height: current_frame.draw_image.image_extent.height,
                width: current_frame.draw_image.image_extent.width,
            };

            image_util::copy_image_to_image(
                &self.device_info.logical_device,
                &current_frame.command_buffer,
                current_frame.draw_image.image,
                self.swapchain_info.swapchain_images[image_index as usize],
                draw_extend,
                self.swapchain_info.swapchain_extent,
            );
            image_util::transition_image_layout(
                &self.device_info,
                &current_frame.command_buffer,
                self.swapchain_info.swapchain_images[image_index as usize],
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                false,
            );
            image_util::transition_image_layout(
                &self.device_info,
                &current_frame.command_buffer,
                self.swapchain_info.swapchain_images[image_index as usize],
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::PRESENT_SRC_KHR,
                false,
            );

            self.device_info
                .logical_device
                .end_command_buffer(current_frame.command_buffer)
                .expect("failed to end command buffer");
        }

        let command_buffer_submit_info = [vk::CommandBufferSubmitInfo::default()
            .command_buffer(current_frame.command_buffer)
            .device_mask(0)];

        let wait_info = [vk::SemaphoreSubmitInfo::default()
            .semaphore(current_frame.swapchain_semaphore)
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT_KHR)
            .device_index(0)
            .value(1)];

        let signal_info = [vk::SemaphoreSubmitInfo::default()
            .semaphore(current_frame.render_semaphore)
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
                    current_frame.render_fence,
                )
                .expect("Unable to submit draw command buffer");
        }

        let render_semaphores = [current_frame.render_semaphore];
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

        self.frame_manager.advance_frame();
    }

    fn render_lighting(&self) {
        let current_frame = self.frame_manager.get_current_frame();

        let color_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(current_frame.draw_image.image_view)
            .image_layout(vk::ImageLayout::GENERAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE);

        let color_attachments = [color_attachment];
        let begin_render_info = vk::RenderingInfo::default()
            .render_area(Rect2D {
                extent: self.swapchain_info.swapchain_extent,
                offset: vk::Offset2D { x: 0, y: 0 },
            })
            .layer_count(1)
            .color_attachments(&color_attachments);

        unsafe {
            self.device_info
                .logical_device
                .cmd_begin_rendering(current_frame.command_buffer, &begin_render_info)
        }

        let width = self.swapchain_info.swapchain_extent.width as f32;
        let height = self.swapchain_info.swapchain_extent.height as f32;
        self.set_viewport_scissor(width, height);

        unsafe {
            self.device_info.logical_device.cmd_bind_pipeline(
                current_frame.command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.frame_manager.lighting_pipeline.pipelines[0],
            );

            self.device_info.logical_device.cmd_bind_descriptor_sets(
                current_frame.command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.frame_manager.lighting_pipeline.pipeline_layout,
                0,
                &[current_frame.descriptor_lighting_set],
                &[],
            );

            self.device_info
                .logical_device
                .cmd_draw(current_frame.command_buffer, 3, 1, 0, 0);

            self.device_info
                .logical_device
                .cmd_end_rendering(current_frame.command_buffer);
        }
    }

    fn render_scene(&self) {
        let current_frame = self.frame_manager.get_current_frame();

        let color_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(current_frame.albedo_image.image_view)
            .image_layout(vk::ImageLayout::GENERAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE);

        let normal_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(current_frame.normal_image.image_view)
            .image_layout(vk::ImageLayout::GENERAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE);

        let pos_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(current_frame.pos_image.image_view)
            .image_layout(vk::ImageLayout::GENERAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE);

        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(current_frame.depth_image.image_view)
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            });

        let color_attachments = [color_attachment, normal_attachment, pos_attachment];
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
                .cmd_begin_rendering(current_frame.command_buffer, &begin_render_info)
        }

        let width = self.swapchain_info.swapchain_extent.width as f32;
        let height = self.swapchain_info.swapchain_extent.height as f32;
        self.set_viewport_scissor(width, height);

        unsafe {
            self.device_info.logical_device.cmd_bind_pipeline(
                current_frame.command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.frame_manager.gbuffer_pipeline.pipelines[0],
            );
        }

        for (i, gpu_mesh) in self.gpu_mesh_data.iter().enumerate() {
            unsafe {
                self.device_info.logical_device.cmd_bind_descriptor_sets(
                    current_frame.command_buffer,
                    PipelineBindPoint::GRAPHICS,
                    self.frame_manager.gbuffer_pipeline.pipeline_layout,
                    0,
                    &[current_frame.descriptor_gbuffer_set],
                    &[i as u32 * self.frame_manager.model_ubo_alignment as u32],
                )
            }

            unsafe {
                self.device_info.logical_device.cmd_bind_index_buffer(
                    current_frame.command_buffer,
                    gpu_mesh.index_buffer.buffer,
                    0,
                    vk::IndexType::UINT32,
                );

                self.device_info.logical_device.cmd_bind_vertex_buffers(
                    current_frame.command_buffer,
                    0,
                    &[gpu_mesh.vertex_buffer.buffer],
                    &[0],
                );

                self.device_info.logical_device.cmd_draw_indexed(
                    current_frame.command_buffer,
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
                .cmd_end_rendering(current_frame.command_buffer);
        }
    }

    fn render_cascade_shadow_map(&self) {
        let current_frame = self.frame_manager.get_current_frame();

        for cascade_index in 0..SHADOW_MAP_CASCADE_COUNT {
            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(
                    current_frame.shadow_cascades[cascade_index]
                        .shadow_depth_image
                        .image_view,
                )
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                });

            let begin_render_info = vk::RenderingInfo::default()
                .render_area(Rect2D {
                    extent: Extent2D {
                        width: 4096,
                        height: 4096,
                    },
                    offset: vk::Offset2D { x: 0, y: 0 },
                })
                .layer_count(1)
                .depth_attachment(&depth_attachment);

            unsafe {
                self.device_info
                    .logical_device
                    .cmd_begin_rendering(current_frame.command_buffer, &begin_render_info)
            }

            self.set_viewport_scissor(4096.0, 4096.0);

            unsafe {
                self.device_info.logical_device.cmd_bind_pipeline(
                    current_frame.command_buffer,
                    PipelineBindPoint::GRAPHICS,
                    self.frame_manager.shadow_map_pipeline.pipelines[0],
                );
            }

            for (_i, gpu_mesh) in self.gpu_mesh_data.iter().enumerate() {
                unsafe {
                    self.device_info.logical_device.cmd_bind_descriptor_sets(
                        current_frame.command_buffer,
                        PipelineBindPoint::GRAPHICS,
                        self.frame_manager.shadow_map_pipeline.pipeline_layout,
                        0,
                        &[current_frame.shadow_map_set],
                        &[],
                    )
                }

                let push = CascadeShadowPushConsts {
                    pos: [0.0, 0.0, 0.0, 0.0],
                    index: cascade_index as u32,
                };

                let x = bincode::serialize(&push).expect("");

                unsafe {
                    self.device_info.logical_device.cmd_push_constants(
                        current_frame.command_buffer,
                        self.frame_manager.shadow_map_pipeline.pipeline_layout,
                        ShaderStageFlags::VERTEX,
                        0,
                        &x,
                    );

                    self.device_info.logical_device.cmd_bind_index_buffer(
                        current_frame.command_buffer,
                        gpu_mesh.index_buffer.buffer,
                        0,
                        vk::IndexType::UINT32,
                    );

                    self.device_info.logical_device.cmd_bind_vertex_buffers(
                        current_frame.command_buffer,
                        0,
                        &[gpu_mesh.vertex_buffer.buffer],
                        &[0],
                    );

                    self.device_info.logical_device.cmd_draw_indexed(
                        current_frame.command_buffer,
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
                    .cmd_end_rendering(current_frame.command_buffer);
            }
        }
    }

    fn set_viewport_scissor(&self, width: f32, height: f32) {
        let current_frame = self.frame_manager.get_current_frame();

        let viewport = vk::Viewport {
            x: 0.0f32,
            y: 0.0f32,
            width,
            height,
            min_depth: 0f32,
            max_depth: 1f32,
        };

        let scissor = Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: Extent2D {
                width: width as u32,
                height: height as u32,
            },
        };

        unsafe {
            self.device_info.logical_device.cmd_set_viewport(
                current_frame.command_buffer,
                0,
                &[viewport],
            );

            self.device_info.logical_device.cmd_set_scissor(
                current_frame.command_buffer,
                0,
                &[scissor],
            );
        }
    }

    fn update_camera(&mut self) {
        let view = self.camera.get_view_matrix();
        let projection = self.camera.get_projection_matrix();

        let current_frame = self.frame_manager.get_mut_current_frame();

        let ubo = CameraMvpUbo {
            view,
            proj: projection,
        };

        current_frame.update_camera_mvp_buffer(ubo);
    }

    fn update_world(&mut self) {
        let current_frame = self.frame_manager.get_mut_current_frame();

        let world_model_data = self
            .gpu_mesh_data
            .iter()
            .map(|data| ModelDynamicUbo {
                model: data.world_model,
            })
            .collect::<Vec<_>>();

        current_frame.update_model_dynamic_buffer(world_model_data);

        let mapped_memory_range = vk::MappedMemoryRange::default()
            .memory(current_frame.model_dynamic_buffer.buffer_memory)
            .size(self.frame_manager.model_ubo_alignment);

        unsafe {
            self.device_info
                .logical_device
                .flush_mapped_memory_ranges(&[mapped_memory_range])
                .expect("failed to flush mapped memory range");
        }
    }

    fn update_lighting_memory(&mut self) {
        let current_frame = self.frame_manager.get_mut_current_frame();
        current_frame.update_lighting_buffer(LightingUbo {
            light_direction: vec3_to_vec4(&glm::normalize(&vec3(0.5, 1.0, 0.5))),

            // w is intensity
            light_color: vec4(1.0, 1.0, 1.0, 1.0),
            ambient_light: vec4(0.1, 0.1, 0.1, 0.2),
            cascade_depths: vec4(
                self.cascades[0].cascade_depth,
                self.cascades[1].cascade_depth,
                self.cascades[2].cascade_depth,
                1.0,
            ),
        })
    }

    fn update_cascade_memory(&mut self) {
        let current_frame = self.frame_manager.get_mut_current_frame();

        let shadow_cascades = self
            .cascades
            .iter()
            .take(3)
            .map(|casc| CascadeShadowUbo {
                cascade_view_proj: casc.cascade_view_proj,
            })
            .collect::<Vec<_>>();

        current_frame.update_shadow_map_buffer(shadow_cascades);
    }

    pub fn update_cascades(&mut self) {
        let lambda = 0.9;
        let mut splits = [0.0; SHADOW_MAP_CASCADE_COUNT + 1];

        for i in 0..=SHADOW_MAP_CASCADE_COUNT {
            let near = self.camera.near_clip;
            let far = self.camera.far_clip;
            let idm = i as f32 / SHADOW_MAP_CASCADE_COUNT as f32;

            let log = near * (far / near).powf(idm);
            let uniform = near + (far - near) * idm;

            splits[i] = log * lambda + uniform * (1.0 - lambda);
        }

        let shadow_map_resolution = 4096.0; // Shadow map resolution

        for i in 0..SHADOW_MAP_CASCADE_COUNT {
            let split_start = splits[i];
            let split_end = splits[i + 1];

            let frustum_corners = [
                vec3(-1.0, -1.0, 0.0),
                vec3(1.0, -1.0, 0.0),
                vec3(1.0, 1.0, 0.0),
                vec3(-1.0, 1.0, 0.0),
                vec3(-1.0, -1.0, 1.0),
                vec3(1.0, -1.0, 1.0),
                vec3(1.0, 1.0, 1.0),
                vec3(-1.0, 1.0, 1.0),
            ];

            let mut corners_world = [Vec3::zeros(); 8];

            let projection_matrix = self
                .camera
                .get_projection_matrix_with_splits(split_start, split_end);
            let inv_projection_view =
                glm::inverse(&(projection_matrix * self.camera.get_view_matrix()));
            for (j, corner) in frustum_corners.iter().enumerate() {
                let corner_world = inv_projection_view * vec4(corner.x, corner.y, corner.z, 1.0);
                corners_world[j] = vec3(
                    corner_world.x / corner_world.w,
                    corner_world.y / corner_world.w,
                    corner_world.z / corner_world.w,
                );
            }

            let mut frustum_center = Vec3::zeros();
            for corner in &corners_world {
                frustum_center += *corner;
            }
            frustum_center /= corners_world.len() as f32;

            let radius = corners_world
                .iter()
                .map(|corner| glm::distance(corner, &frustum_center))
                .fold(0.0, f32::max);

            let max_extents = vec3(radius, radius, radius);
            let min_extents = -max_extents;

            let texel_size = (max_extents.x - min_extents.x) / shadow_map_resolution;
            let snapped_min_extents = glm::floor(&(min_extents / texel_size)) * texel_size;
            let snapped_max_extents = glm::ceil(&(max_extents / texel_size)) * texel_size;

            let light_dir = glm::normalize(&vec3(0.5, 1.0, 0.5)); // Assuming directional light

            let light_position: Vec3 = frustum_center + light_dir * radius;

            let mut up = vec3(0.0, 1.0, 0.0);
            if glm::normalize(&light_position).y > 0.98 {
                up = vec3(0.0, 1.0, 0.0);
            }

            let light_view = glm::look_at(&(light_position), &frustum_center, &up);

            let mut light_projection = glm::ortho(
                snapped_min_extents.x,
                snapped_max_extents.x,
                snapped_min_extents.y,
                snapped_max_extents.y,
                snapped_min_extents.z,
                snapped_max_extents.z - snapped_min_extents.z,
            );

            light_projection[(1, 1)] *= -1.0;

            let normalized_start = split_start;
            let normalized_end = split_end;
            let dist = normalized_end - normalized_start;

            self.cascades[i] = Cascade {
                cascade_view_proj: light_projection * light_view,
                cascade_depth: normalized_start + dist,
            }
        }
    }

    fn create_image_view(
        device_info: &DeviceInfo,
        image: vk::Image,
        format: vk::Format,
        image_aspect_flags: ImageAspectFlags,
    ) -> ImageView {
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

    fn create_texture_image_view(device_info: &DeviceInfo, image: vk::Image) -> ImageView {
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
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(Extent3D {
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
        instance: &Instance,
    ) -> (vk::Image, vk::DeviceMemory) {
        let dyn_image = image::open(".\\resources\\textures\\texture.png").unwrap();
        let image_width = dyn_image.width();
        let image_height = dyn_image.height();

        let image_size =
            (mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;

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
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
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

        let (image, device_memory) = image_util::create_image(
            device_info,
            instance,
            dyn_image.width(),
            dyn_image.height(),
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            MemoryPropertyFlags::DEVICE_LOCAL,
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

    fn create_buffer(
        instance: &Instance,
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
            .memory_type_index(utils::find_memory_type(
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

    fn copy_buffer(
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: u64,
        device_info: &DeviceInfo,
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
        instance: &Instance,
        device_info: &DeviceInfo,
        indices: &[u32],
    ) -> AllocatedBuffer {
        let buffer_size = mem::size_of_val(indices) as vk::DeviceSize;

        let (x, y) = Self::create_buffer(
            instance,
            device_info,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
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
            MemoryPropertyFlags::DEVICE_LOCAL,
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
        instance: &Instance,
        device_info: &DeviceInfo,
        vertices: &[Vertex],
    ) -> AllocatedBuffer {
        let buffer_size = mem::size_of_val(vertices) as u64;

        let (x, y) = Self::create_buffer(
            instance,
            device_info,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
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
            MemoryPropertyFlags::DEVICE_LOCAL,
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

    fn create_instance(entry: &ash::Entry, window: &Window) -> Instance {
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
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
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
    ) -> Vec<ImageView> {
        let mut image_views = vec![];

        for swapchain_image in swapchain_info.swapchain_images.clone() {
            let image_view = AllocatedImage::create_image_view(
                device_info,
                &swapchain_image,
                swapchain_info.swapchain_image_format.format,
                ImageAspectFlags::COLOR,
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
            SwapchainInfo::new(&self.instance, &self.device_info, &self.surface_info);
        self.image_views = Self::create_image_views(&self.swapchain_info, &self.device_info);
    }

    fn cleanup_swapchain(&mut self) {
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

    pub fn get_cascade_frustum_corners(&self, cascade_matrix: &Mat4, color: Vec3) -> Vec<Vertex> {
        let inverse_matrix = glm::inverse(cascade_matrix);

        // NDC corners of the unit cube
        let ndc_corners = [
            vec3(-1.0, -1.0, -1.0),
            vec3(1.0, -1.0, -1.0),
            vec3(1.0, 1.0, -1.0),
            vec3(-1.0, 1.0, -1.0),
            vec3(-1.0, -1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
        ];

        // Transform corners to world space
        let frustum_corners_world: Vec<Vec3> = ndc_corners
            .iter()
            .map(|corner| {
                let world_space = inverse_matrix * vec4(corner.x, corner.y, corner.z, 1.0);
                vec3(world_space.x, world_space.y, world_space.z) / world_space.w
            })
            .collect();

        // Create lines connecting the corners
        let mut lines = Vec::new();

        let line_pairs = [
            // Near plane
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            // Far plane
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            // Connecting lines
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        for &(start, end) in &line_pairs {
            lines.push(Vertex {
                pos: frustum_corners_world[start],
                color,
                tex_coord: vec2(0.0, 0.0),
                normal: vec3(0.0, 0.0, 0.0),
                ..Vertex::default()
            });
            lines.push(Vertex {
                pos: frustum_corners_world[end],
                color,
                tex_coord: vec2(0.0, 0.0),
                normal: vec3(0.0, 0.0, 0.0),
                ..Vertex::default()
            });
        }

        lines
    }
}
