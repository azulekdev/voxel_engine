#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Dirt,
    Stone,
    Grass,
    Water,
    OakLog,
    Leaves,
}

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub block_type: BlockType,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self { block_type }
    }

    pub fn is_active(&self) -> bool {
        self.block_type != BlockType::Air
    }

    pub fn is_transparent(&self) -> bool {
        matches!(
            self.block_type,
            BlockType::Air | BlockType::Water | BlockType::Leaves
        )
    }

    pub fn is_water(&self) -> bool {
        matches!(self.block_type, BlockType::Water)
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            block_type: BlockType::Air,
        }
    }
}
