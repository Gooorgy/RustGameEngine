use crate::backend_impl::vulkan_backend::VulkanBackend;
use crate::buffer::{BufferDesc, BufferHandle, BufferUsageFlags};
use crate::image::{GpuImageHandle, ImageAspect, ImageDesc, ImageUsageFlags, TextureFormat};
use crate::memory::MemoryHint;
use common::{ImageData, ImageHandle, MeshData, MeshHandle, Vertex};
use std::collections::HashMap;
use std::mem;

#[derive(Copy, Clone)]
pub struct GpuMeshData {
    pub vertex_buffer: BufferHandle,
    pub index_buffer: BufferHandle,
    pub index_count: usize,
}

pub struct ResourceManager {
    pub mesh_data: HashMap<MeshHandle, GpuMeshData>,
    pub images: HashMap<ImageHandle, GpuImageHandle>,
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            mesh_data: HashMap::new(),
            images: HashMap::new(),
        }
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    pub fn get_or_create_mesh(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        handle: MeshHandle,
        mesh: &MeshData,
    ) -> GpuMeshData {
        let mesh_data = self.mesh_data.get_mut(&handle);
        if let Some(mesh_data) = mesh_data {
            return *mesh_data;
        }

        self.upload_mesh(vulkan_backend, mesh, handle)
    }

    fn upload_mesh(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        mesh: &MeshData,
        handle: MeshHandle,
    ) -> GpuMeshData {
        let vertices = mesh.vertices.as_slice();
        let indices = mesh.indices.as_slice();

        let vertex_buffer_size = mem::size_of_val(vertices);
        let index_buffer_size = mem::size_of_val(indices);
        let vertex_buffer_handle = vulkan_backend.create_buffer(
            BufferDesc {
                usage: BufferUsageFlags::VERTEX_BUFFER,
                memory_hint: MemoryHint::GPUOnly,
                size: vertex_buffer_size,
            },
            Some(vertices),
        );
        let index_buffer_handle = vulkan_backend.create_buffer(
            BufferDesc {
                usage: BufferUsageFlags::INDEX_BUFFER,
                memory_hint: MemoryHint::GPUOnly,
                size: index_buffer_size,
            },
            Some(indices),
        );

        let mesh_data = GpuMeshData {
            vertex_buffer: vertex_buffer_handle,
            index_buffer: index_buffer_handle,
            index_count: indices.len(),
        };
        self.mesh_data.insert(handle, mesh_data);

        mesh_data
    }

    pub fn get_or_create_image(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        handle: ImageHandle,
        data: &ImageData,
    ) -> GpuImageHandle {
        let images = self.images.get_mut(&handle);
        if let Some(images) = images {
            return *images;
        }

        let image_desc = ImageDesc {
            width: data.width,
            height: data.height,
            usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            aspect: ImageAspect::Color,
            array_layers: 0,
            is_cubemap: false,
            mip_levels: 0,
            format: TextureFormat::R8g8b8a8Unorm,
            clear_value: None,
            depth: 1,
        };

        let image_handle = vulkan_backend.create_image(image_desc);
        vulkan_backend.update_image_data(image_handle, data.pixels.as_ref());
        self.images.insert(handle, image_handle);

        image_handle
    }
}
