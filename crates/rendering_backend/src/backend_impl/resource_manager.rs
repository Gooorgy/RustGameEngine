use crate::backend_impl::vulkan_backend::VulkanBackend;
use crate::buffer::{BufferDesc, BufferHandle, BufferUsageFlags};
use crate::image::{GpuImageHandle, ImageAspect, ImageDesc, ImageUsageFlags, TextureFormat};
use crate::memory::MemoryHint;
use crate::vertex::Vertex;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct MeshData {
    pub vertex_buffer: BufferHandle,
    pub index_buffer: BufferHandle,
    pub index_count: usize,
}

pub struct ResourceManager {
    pub mesh_data: HashMap<u64, MeshData>,
    pub images: HashMap<u64, GpuImageHandle>,
}

#[derive(Clone)]
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

    pub fn upload_meshes(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        meshes: HashMap<u64, Rc<Mesh>>,
    ) {
        for (asset_id, mesh) in meshes {
            let vertices = mesh.vertices.as_slice();
            let indices = mesh.indices.as_slice();

            let vertex_buffer_size = mem::size_of_val(vertices);
            let index_buffer_size = mem::size_of_val(indices);
            println!("uploading vertex buffer");
            let vertex_buffer_handle = vulkan_backend.create_buffer(
                BufferDesc {
                    usage: BufferUsageFlags::VERTEX_BUFFER,
                    memory_hint: MemoryHint::GPUOnly,
                    size: vertex_buffer_size,
                },
                Some(vertices),
            );
            println!("uploading index buffer");
            let index_buffer_handle = vulkan_backend.create_buffer(
                BufferDesc {
                    usage: BufferUsageFlags::INDEX_BUFFER,
                    memory_hint: MemoryHint::GPUOnly,
                    size: index_buffer_size,
                },
                Some(indices),
            );

            self.mesh_data.insert(
                asset_id,
                MeshData {
                    vertex_buffer: vertex_buffer_handle,
                    index_buffer: index_buffer_handle,
                    index_count: indices.len(),
                },
            );
        }
    }

    pub fn upload_images(&mut self, vulkan_backend: &mut VulkanBackend) {}

    pub fn get_by_handle(&self, vulkan_backend: &mut VulkanBackend, asset_id: u64) -> &MeshData {
        self.mesh_data.get(&asset_id).unwrap()
    }

    pub fn get_or_create_mesh(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        asset_id: u64,
        mesh: Rc<Mesh>,
    ) -> MeshData {
        let mesh_data = self.mesh_data.get_mut(&asset_id);
        if let Some(mesh_data) = mesh_data {
            return *mesh_data;
        }

        self.upload_mesh(vulkan_backend, mesh, asset_id)
    }

    fn upload_mesh(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        mesh: Rc<Mesh>,
        index: u64,
    ) -> MeshData {
        let vertices = mesh.vertices.as_slice();
        let indices = mesh.indices.as_slice();

        let vertex_buffer_size = mem::size_of_val(vertices);
        let index_buffer_size = mem::size_of_val(indices);
        println!("uploading vertex buffer");
        let vertex_buffer_handle = vulkan_backend.create_buffer(
            BufferDesc {
                usage: BufferUsageFlags::VERTEX_BUFFER,
                memory_hint: MemoryHint::GPUOnly,
                size: vertex_buffer_size,
            },
            Some(vertices),
        );
        println!("uploading index buffer");
        let index_buffer_handle = vulkan_backend.create_buffer(
            BufferDesc {
                usage: BufferUsageFlags::INDEX_BUFFER,
                memory_hint: MemoryHint::GPUOnly,
                size: index_buffer_size,
            },
            Some(indices),
        );

        let mesh_data = MeshData {
            vertex_buffer: vertex_buffer_handle,
            index_buffer: index_buffer_handle,
            index_count: indices.len(),
        };
        self.mesh_data.insert(index, mesh_data);

        mesh_data
    }

    pub fn get_or_create_image(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        asset_id: u64,
        data: Rc<Vec<u8>>,
        width: u32,
        height: u32,
    ) -> GpuImageHandle {
        let images = self.images.get_mut(&asset_id);
        if let Some(images) = images {
            return *images;
        }

        let image_desc = ImageDesc {
            width,
            height,
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
        vulkan_backend.update_image_data(image_handle, data.as_ref());
        self.images.insert(asset_id, image_handle);

        image_handle
    }
}
