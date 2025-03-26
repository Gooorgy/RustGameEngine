use std::collections::HashMap;
use crate::terrain::terrain_material::{BlockData, BlockShader};

pub type BlockNameSpace = &'static str;

/*impl From<BlockType> for BlockNameSpace {
    fn from(value: BlockType) -> Self {
        Self(value.as_namespace())
    }
}*/
pub type BlockRegistry = HashMap<BlockNameSpace, BlockDefinition>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BlockType(pub BlockNameSpace);
impl BlockType {
    pub const NONE: Self = Self("Block::None");
    pub const GRASS: Self = Self("Block::Grass");
    pub const STONE: Self = Self("Block::Stone");
}

impl BlockType {
    pub const fn as_namespace(&self) -> BlockNameSpace {
        self.0
    }
}
pub struct None {}

pub struct BlockDefinition {
    pub block_shader: BlockShader,
    pub is_solid: bool,
}

impl None {
    const BLOCK_ID: BlockType = BlockType::NONE;
    const RENDER_TYPE: BlockShader = BlockShader::None;
    const IS_SOLID: bool = false;
}

impl BlockData for None {
    fn block_id(&self) -> BlockNameSpace {
        BlockType::GRASS.as_namespace()
    }

    fn is_solid(&self) -> bool {
        Self::IS_SOLID
    }

    fn render_type(&self) -> BlockShader {
        Self::RENDER_TYPE
    }
}

pub struct Grass {}

impl Grass {
    const BLOCK_ID: BlockType = BlockType::GRASS;
    const RENDER_TYPE: BlockShader = BlockShader::Opaque;
    const IS_SOLID: bool = true;
}

impl BlockData for Grass {
    fn block_id(&self) -> BlockNameSpace {
        Self::BLOCK_ID.as_namespace()
    }

    fn is_solid(&self) -> bool {
        Self::IS_SOLID
    }

    fn render_type(&self) -> BlockShader {
        Self::RENDER_TYPE
    }
}

pub struct Stone {}

impl Stone {
    const BLOCK_ID: BlockType = BlockType::STONE;
    const RENDER_TYPE: BlockShader = BlockShader::Opaque;
    const IS_SOLID: bool = true;
}
impl BlockData for Stone {
    fn block_id(&self) -> BlockNameSpace {
        Self::BLOCK_ID.as_namespace()
    }

    fn is_solid(&self) -> bool {
        Self::IS_SOLID
    }

    fn render_type(&self) -> BlockShader {
        Self::RENDER_TYPE
    }
}
