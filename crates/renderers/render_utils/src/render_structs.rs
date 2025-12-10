use crate::renderers::geometry_renderer::GeometryRenderer;
use glm::Mat4;
use rendering_backend::backend_impl::resource_manager::MeshData;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::buffer::{BufferDesc, BufferHandle, BufferUsageFlags};
use rendering_backend::camera::CameraMvpUbo;
use rendering_backend::image::{
    ImageAspect, ImageDesc, ImageHandle, ImageUsageFlags, TextureFormat,
};
use rendering_backend::memory::MemoryHint;
use std::mem;
use std::mem::size_of;

pub const MAX_MESHES: usize = 1000;

pub struct FrameImages {
    gbuffer_albedo: ImageHandle,
    gbuffer_normal: ImageHandle,
    gbuffer_depth: ImageHandle,
}

pub struct Renderer {
    geometry_renderer: GeometryRenderer,
    scene_data: Vec<MeshData>,
    frame_images: FrameImages,
    camera_buffer: BufferHandle,
    model_storage_buffer: BufferHandle,
}

pub struct ResolutionSettings {
    pub window_resolution: Resolution,
    // shadow_resolution: Resolution,
}

pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub fn new(
        vulkan_backend: &mut VulkanBackend,
        resolution_settings: ResolutionSettings,
    ) -> Renderer {
        let frame_images = Self::create_frame_images(vulkan_backend, resolution_settings);
        let buffer_size = mem::size_of::<CameraMvpUbo>();

        let camera_buffer = vulkan_backend.create_buffer::<CameraMvpUbo>(
            BufferDesc {
                size: buffer_size,
                usage: BufferUsageFlags::UNIFORM,
                memory_hint: MemoryHint::CPUWritable,
            },
            None,
        );

        let model_storage_buffer = vulkan_backend.create_buffer::<CameraMvpUbo>(
            BufferDesc {
                size: size_of::<Mat4>() * MAX_MESHES,
                memory_hint: MemoryHint::CPUWritable,
                usage: BufferUsageFlags::STORAGE,
            },
            None,
        );

        let geometry_renderer = GeometryRenderer::new(
            vulkan_backend,
            frame_images.gbuffer_albedo,
            frame_images.gbuffer_depth,
            camera_buffer,
            model_storage_buffer,
        );

        Renderer {
            geometry_renderer,
            scene_data: Vec::new(),
            frame_images,
            camera_buffer,
            model_storage_buffer,
        }
    }

    pub fn update_camera(&self, vulkan_backend: &mut VulkanBackend, camera_mvp_ubo: CameraMvpUbo) {
        vulkan_backend.update_buffer(self.camera_buffer, &[camera_mvp_ubo])
    }

    pub fn update_scene(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        mesh_data: &[(Mat4, MeshData)],
    ) {
        let mut scene_data = vec![];
        let mut matrices = vec![];
        for (model, mesh) in mesh_data {
            scene_data.push(*mesh);
            matrices.push(*model);
        }

        self.scene_data = scene_data;

        vulkan_backend.update_buffer(self.model_storage_buffer, matrices.as_slice());
    }

    pub fn draw_frame(&self, vulkan_backend: &mut VulkanBackend) {
        vulkan_backend.begin_frame();

        self.geometry_renderer.render(
            vulkan_backend,
            &self.scene_data,
            self.frame_images.gbuffer_albedo,
            self.frame_images.gbuffer_depth,
        );

        vulkan_backend.end_frame(self.frame_images.gbuffer_albedo);
    }

    fn create_frame_images(
        vulkan_backend: &mut VulkanBackend,
        resolution_settings: ResolutionSettings,
    ) -> FrameImages {
        let window_resolution = resolution_settings.window_resolution;
        let gbuffer_albedo = vulkan_backend.create_image(ImageDesc {
            width: window_resolution.width,
            height: window_resolution.height,
            depth: 1,
            format: TextureFormat::R16g16b16a16Float,
            clear_value: None,
            array_layers: 1,
            is_cubemap: false,
            mip_levels: 1,
            aspect: ImageAspect::Color,
            usage: ImageUsageFlags::COLOR_ATTACHMENT
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST
                | ImageUsageFlags::SAMPLED
                | ImageUsageFlags::STORAGE,
        });
        let gbuffer_normal = vulkan_backend.create_image(ImageDesc {
            width: window_resolution.width,
            height: window_resolution.height,
            depth: 1,
            format: TextureFormat::R16g16b16a16Float,
            clear_value: None,
            array_layers: 1,
            is_cubemap: false,
            mip_levels: 1,
            aspect: ImageAspect::Color,
            usage: ImageUsageFlags::COLOR_ATTACHMENT
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST
                | ImageUsageFlags::SAMPLED
                | ImageUsageFlags::STORAGE,
        });
        let gbuffer_depth = vulkan_backend.create_image(ImageDesc {
            width: window_resolution.width,
            height: window_resolution.height,
            depth: 1,
            format: TextureFormat::D32Float,
            clear_value: None,
            array_layers: 1,
            is_cubemap: false,
            mip_levels: 1,
            aspect: ImageAspect::Depth,
            usage: ImageUsageFlags::DEPTH_ATTACHMENT
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST
                | ImageUsageFlags::SAMPLED
                | ImageUsageFlags::STORAGE,
        });

        FrameImages {
            gbuffer_albedo,
            gbuffer_normal,
            gbuffer_depth,
        }
    }
}

// let albedo_image = AllocatedImage::new(
// device_info,
// instance,
// image_width,
// image_height,
// Format::R16G16B16A16_SFLOAT,
// ImageAspectFlags::COLOR,
// vk::ImageTiling::OPTIMAL,
// vk::ImageUsageFlags::TRANSFER_DST
// | vk::ImageUsageFlags::TRANSFER_SRC
// | vk::ImageUsageFlags::STORAGE
// | vk::ImageUsageFlags::COLOR_ATTACHMENT
// | vk::ImageUsageFlags::SAMPLED,
// MemoryPropertyFlags::DEVICE_LOCAL,
// );
//
// let normal_image = AllocatedImage::new(
// device_info,
// instance,
// image_width,
// image_height,
// Format::R16G16B16A16_SFLOAT,
// ImageAspectFlags::COLOR,
// vk::ImageTiling::OPTIMAL,
// vk::ImageUsageFlags::TRANSFER_DST
// | vk::ImageUsageFlags::TRANSFER_SRC
// | vk::ImageUsageFlags::COLOR_ATTACHMENT
// | vk::ImageUsageFlags::SAMPLED,
// MemoryPropertyFlags::DEVICE_LOCAL,
// );
//
// let depth_image = AllocatedImage::new(
// device_info,
// instance,
// image_width,
// image_height,
// Format::D32_SFLOAT,
// ImageAspectFlags::DEPTH,
// vk::ImageTiling::OPTIMAL,
// vk::ImageUsageFlags::TRANSFER_DST
// | vk::ImageUsageFlags::TRANSFER_SRC
// | vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
// | vk::ImageUsageFlags::SAMPLED,
// MemoryPropertyFlags::DEVICE_LOCAL,
// );
