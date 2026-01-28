use crate::block::Block;
use crate::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use std::collections::HashMap;

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    // FIXED coordinate conversion for negative values
    fn world_to_chunk_coords(world_x: i32, world_z: i32) -> ((i32, i32), (usize, usize)) {
        let chunk_x = if world_x >= 0 {
            world_x / CHUNK_SIZE as i32
        } else {
            (world_x + 1) / CHUNK_SIZE as i32 - 1
        };

        let chunk_z = if world_z >= 0 {
            world_z / CHUNK_SIZE as i32
        } else {
            (world_z + 1) / CHUNK_SIZE as i32 - 1
        };

        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;

        ((chunk_x, chunk_z), (local_x, local_z))
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return None;
        }

        let ((chunk_x, chunk_z), (local_x, local_z)) = Self::world_to_chunk_coords(x, z);

        if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_z)) {
            if let Some(block) = chunk.get_block(local_x, y as usize, local_z) {
                return Some(*block);
            }
        }
        None
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return;
        }

        let ((chunk_x, chunk_z), (local_x, local_z)) = Self::world_to_chunk_coords(x, z);

        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
            chunk.set_block(local_x, y as usize, local_z, block.block_type);
        }
    }
}
