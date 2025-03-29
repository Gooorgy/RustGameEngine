use crate::terrain::blocks::blocks::BlockDefinition;
use crate::terrain::terrain_material::BlockShader;

/*macro_rules! block_defs {
    ($($block:expr => $def:expr),* $(,)?) => {{
        let mut map = HashMap::new();
        $( map.insert($block.0, $def); )*
        map
    }};
}

pub const BLOCK_DEFINITIONS: HashMap<BlockNameSpace, BlockDefinition> = block_defs! {
    BlockType::GRASS => GRASS,
    BlockType::STONE => STONE,
};
*/

pub const NONE: BlockDefinition = BlockDefinition {
    block_shader: BlockShader::Opaque,
    is_solid: false,
};

pub const GRASS: BlockDefinition = BlockDefinition {
    block_shader: BlockShader::Opaque,
    is_solid: true,
};

pub const STONE: BlockDefinition = BlockDefinition {
    block_shader: BlockShader::Opaque,
    is_solid: true,
};
