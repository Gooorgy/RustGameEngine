use crate::terrain::blocks;
use crate::terrain::blocks::blocks::{BlockNameSpace, BlockType};
use crate::vulkan_render::scene::ImageResource;

pub struct TerrainMaterial {
    block_shader: BlockShader,
}

pub enum BlockShader {
    None,
    Opaque,
    Transparent,
    //Emissive,
    Custom(CustomShaderData),
}

pub struct OpaqueShaderData{
    pub texture: ImageResource,
    pub normal: Option<ImageResource>,
    pub specular: Option<ImageResource>,
}

pub struct TransparentShaderData {
    // TODO
}

pub struct CustomShaderData {
    // TODO
}

struct OpaqueBlockShader {
    block_textures: Vec<BlockTexture>,
}

struct TransparentBlockShader {}

struct BlockTexture {
    texture: ImageResource,
    normal: Option<ImageResource>,
    faces: Vec<BlockFace>,
}

pub enum BlockFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

pub trait BlockData {
    fn block_id(&self) -> BlockNameSpace;

    fn is_solid(&self) -> bool;

    fn render_type(&self) -> BlockShader;
}

pub struct ChunkData {
    voxels: Voxels,
}

struct Voxels(Vec<Vec<Vec<BlockNameSpace>>>);

#[derive(Copy, Clone)]
pub struct VoxelData {
    pub block: BlockNameSpace,
    pub orientation: Option<VoxelOrientation>,
}

#[derive(Copy, Clone)]
pub struct VoxelOrientation {}

impl VoxelData {
    pub fn default() -> VoxelData {
        VoxelData {
            block: BlockType::NONE.as_namespace(),
            orientation: None,
        }
    }

    pub fn from_block_name_space(block_name_space: BlockNameSpace) -> VoxelData {
        VoxelData {
            block: block_name_space,
            orientation: None,
        }
    }
}
