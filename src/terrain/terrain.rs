use crate::terrain::blocks::block_definitions::{GRASS, NONE, STONE};
use crate::terrain::blocks::blocks::{BlockDefinition, BlockNameSpace, BlockType};
use crate::terrain::constants::{
    Face, CHUNK_SIZE, CHUNK_STORAGE_SIZE, FACE_INDICES, FACE_VERTICES,
};
use crate::terrain::terrain_material::VoxelData;
use crate::vulkan_render::renderable::Render;
use crate::vulkan_render::scene::Mesh;
use crate::vulkan_render::structs::{TerrainVertex, Vertex};
use ash::vk::CommandBuffer;
use glm::{vec2, vec3, IVec3, Vec3};
use nalgebra::Vector3;
use std::collections::HashMap;

const VOXEL_SIZE: i32 = 20;
const VOXEL_SIZE_HALF: i32 = VOXEL_SIZE / 2;
const VOXEL_SIZE_HALF_F32: f32 = VOXEL_SIZE_HALF as f32;

pub struct Terrain {
    chunks: Vec<TerrainChunk>,
    block_registry: HashMap<BlockNameSpace, BlockDefinition>,
}

impl Terrain {
    pub fn new(blocks: HashMap<BlockNameSpace, BlockDefinition>) -> Terrain {
        Terrain {
            chunks: Vec::new(),
            block_registry: blocks,
        }
    }

    fn add_voxel_to_stack(&self, block_type: BlockNameSpace) {}
}

impl Render for Terrain {
    fn render(&self, command_buffer: CommandBuffer) {
        for chunk in &self.chunks {
            chunk.render(command_buffer);
        }
    }
}

pub struct TerrainChunk {
    pub chunk_coords: IVec3,
    pub voxel_data: [VoxelData; CHUNK_STORAGE_SIZE],
    pub opaque_mesh: Option<TerrainMesh>,
    //pub transparent_mesh: TerrainMesh,
}

impl Render for TerrainChunk {
    fn render(&self, command_buffer: CommandBuffer) {
        todo!()
    }
}

impl TerrainChunk {
    pub fn new(voxel_data: [VoxelData; CHUNK_STORAGE_SIZE]) -> Self {
        Self {
            chunk_coords: IVec3::new(0, 0, 0),
            voxel_data,
            opaque_mesh: None,
        }
    }

    fn add_face(
        &self,
        face: Face,
        pos: IVec3,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        element_index: &mut u32,
    ) {
        let base_vertices = FACE_VERTICES[face as usize];

        let face_vertices = base_vertices
            .iter()
            .map(|&(mut base_vertex)| {
                let base_pos = base_vertex.pos;
                let relative_pos = vec3(
                    base_pos.x + (pos.x * VOXEL_SIZE) as f32,
                    base_pos.y + (pos.y * VOXEL_SIZE) as f32,
                    base_pos.z + (pos.z * VOXEL_SIZE) as f32,
                );

                *base_vertex.pos = *relative_pos;
                base_vertex
            })
            .collect::<Vec<_>>();

        let base_indices = FACE_INDICES[face as usize];
        let face_indices = base_indices
            .iter()
            .map(|index| index + *element_index)
            .collect::<Vec<_>>();
        *element_index += 4;

        indices.extend_from_slice(face_indices.as_slice());
        vertices.extend_from_slice(face_vertices.as_slice());
    }

    pub fn build_chunk_mesh(&self) -> Mesh {
        let mut axis_columns = [[[0u16; CHUNK_SIZE]; CHUNK_SIZE]; 3];
        let mut face_mask = [[[0u16; CHUNK_SIZE]; CHUNK_SIZE]; 6];

        let BLOCK_DEFINITIONS: HashMap<BlockNameSpace, BlockDefinition> = HashMap::from([
            (BlockType::GRASS.as_namespace(), GRASS),
            (BlockType::STONE.as_namespace(), STONE),
            (BlockType::NONE.as_namespace(), NONE),
        ]);

        // Each axis_column represents:
        // [0] = Y-axis columns (for top/bottom faces) - bits represent X-Z plane
        // [1] = X-axis columns (for left/right faces) - bits represent Y-Z plane
        // [2] = Z-axis columns (for front/back faces) - bits represent X-Y plane

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let index = (x) + (y) * CHUNK_SIZE + (z) * CHUNK_SIZE * CHUNK_SIZE;

                    let voxel_data = &self.voxel_data[index];
                    let is_solid = BLOCK_DEFINITIONS
                        .get(voxel_data.block)
                        .expect("TODO")
                        .is_solid;

                    if is_solid {
                        // Fix axis mapping:
                        axis_columns[0][x][z] |= 1u16 << y; // Y-axis (height)
                        axis_columns[1][z][y] |= 1u16 << x; // X-axis (width)
                        axis_columns[2][x][y] |= 1u16 << z; // Z-axis (depth)
                    }
                }
            }
        }

        for axis in 0..3 {
            for z in 0..CHUNK_SIZE  {
                for x in 0..CHUNK_SIZE  {
                    let column = axis_columns[axis][z][x];

                    // Fix face mask calculation
                    face_mask[2 * axis][z][x] = column & !(column << 1); // Positive face
                    face_mask[2 * axis + 1][z][x] = column & !(column >> 1); // Negative face
                }
            }
        }

        let mut vertices = vec![];
        let mut indices = vec![];
        let mut element_index = 0;

        for face in Face::iter() {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let mut mask = face_mask[face as usize][z][x];
                    while mask != 0 {
                        let y = mask.trailing_zeros();
                        mask &= mask - 1;

                        let voxel_pos = match face {
                            Face::Bottom | Face::Top => IVec3::new(x as i32, y as i32, z as i32),
                            Face::Back | Face::Front => IVec3::new(z as i32, x as i32, y as i32),
                            Face::Left | Face::Right => IVec3::new(y as i32, x as i32, z as i32),
                        };

                        self.add_face(
                            face,
                            voxel_pos,
                            &mut vertices,
                            &mut indices,
                            &mut element_index,
                        );
                    }
                }
            }
        }
        /*        for x in 0..CHUNK_SIZE - 2 {
            for y in 0..CHUNK_SIZE - 2 {
                for z in 0..CHUNK_SIZE - 2 {
                    let index = (x + 1) + (y + 1) * CHUNK_SIZE + (z + 1) * CHUNK_SIZE * CHUNK_SIZE;

                    let voxel_data = &self.voxel_data[index];

                    let voxel_data = &self.voxel_data[index];
                    let is_solid = BLOCK_DEFINITIONS
                        .get(voxel_data.block)
                        .expect("TODO")
                        .is_solid;

                    if(is_solid) {


                    for face in Face::iter() {
                        self.add_face(
                            face,
                            IVec3::new(x as i32, y as i32, z as i32),
                            &mut vertices,
                            &mut indices,
                            &mut element_index,
                        );
                    }
                    }
                }
            }
        }*/

        Mesh { vertices, indices }
    }
}

pub struct TerrainMesh {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
}

struct BinaryFaceStack {}
