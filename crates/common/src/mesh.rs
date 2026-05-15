use crate::handle::Handle;
use nalgebra::{Vector2, Vector3};

#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub color: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
    pub normal: Vector3<f32>,
    pub texture_index: u32,
}

/// A contiguous range of the index buffer that uses a single material slot.
#[derive(Clone, Debug)]
pub struct SubMesh {
    /// Offset (in indices, not bytes) into the mesh's index buffer.
    pub index_offset: u32,
    pub index_count: u32,
}

/// CPU-side mesh data. All submeshes share a single vertex and index buffer.
/// A single-material mesh has exactly one submesh covering all indices.
#[derive(Clone, Debug)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    /// One entry per material slot, in order. Must not be empty.
    pub submeshes: Vec<SubMesh>,
}

pub type MeshHandle = Handle<MeshData>;
