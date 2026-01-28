use crate::block::{Block, BlockType};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub blocks: Box<[[[Block; CHUNK_SIZE]; CHUNK_HEIGHT]; CHUNK_SIZE]>,
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            blocks: Box::new([[[Block::default(); CHUNK_SIZE]; CHUNK_HEIGHT]; CHUNK_SIZE]),
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_type: BlockType) {
        if x < CHUNK_SIZE && y < CHUNK_HEIGHT && z < CHUNK_SIZE {
            self.blocks[x][y][z] = Block::new(block_type);
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        if x < CHUNK_SIZE && y < CHUNK_HEIGHT && z < CHUNK_SIZE {
            Some(&self.blocks[x][y][z])
        } else {
            None
        }
    }
}
