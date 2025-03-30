use crate::terrain::blocks::blocks::{BlockNameSpace, BlockType};
use vulkan_backend::scene::ImageResource;

pub enum BlockShader {
    None,
    Opaque,
    Transparent,
    Custom(CustomShaderData),
}

pub struct OpaqueShaderData {
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
