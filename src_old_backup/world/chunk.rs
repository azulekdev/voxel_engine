use super::block::{Block, BlockType};
use glam::Vec3;

// Chunk dimensions
pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_DEPTH: usize = 16;

pub struct Chunk {
    pub blocks: Box<[[[Block; CHUNK_DEPTH]; CHUNK_HEIGHT]; CHUNK_WIDTH]>,
    #[allow(dead_code)]
    pub position: Vec3,
}

impl Chunk {
    pub fn new(position: Vec3) -> Self {
        // Initialize with default (Air)
        let blocks = Box::new([[[Block::default(); CHUNK_DEPTH]; CHUNK_HEIGHT]; CHUNK_WIDTH]);

        Self { blocks, position }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_type: BlockType) {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            self.blocks[x][y][z] = Block::new(block_type);
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            Some(&self.blocks[x][y][z])
        } else {
            None
        }
    }
}
