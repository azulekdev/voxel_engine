// Block types for the voxel engine
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockType {
    Air,
    Dirt,
    Stone,
    Grass,
    Water,
    OakLog,
    Leaves,
}

// Block with properties
#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub block_type: BlockType,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self { block_type }
    }

    pub fn is_solid(&self) -> bool {
        !matches!(self.block_type, BlockType::Air | BlockType::Water)
    }

    pub fn is_transparent(&self) -> bool {
        matches!(
            self.block_type,
            BlockType::Air | BlockType::Water | BlockType::Leaves
        )
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            block_type: BlockType::Air,
        }
    }
}
