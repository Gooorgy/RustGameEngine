use crate::backend_impl::vulkan_backend::VulkanBackend;
use crate::buffer::{BufferDesc, BufferHandle, BufferUsageFlags};
use crate::memory::MemoryHint;
use crate::vertex::Vertex;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct MeshData {
    pub vertex_buffer: BufferHandle,
    pub index_buffer: BufferHandle,
    pub index_count: usize,
}

pub struct ResourceManager {
    pub mesh_data: HashMap<u64, MeshData>,
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

    pub fn get_by_handle(&self, asset_id: u64) -> &MeshData {
        self.mesh_data.get(&asset_id).unwrap()
    }
}
