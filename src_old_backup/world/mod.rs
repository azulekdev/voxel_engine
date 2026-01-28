pub mod block;
pub mod chunk;
pub mod terrain;

use block::Block;
use chunk::{CHUNK_DEPTH, CHUNK_WIDTH, Chunk};
// use glam::Vec3;
use std::collections::HashMap;
use terrain::WorldGenerator;

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub generator: WorldGenerator,
}

impl World {
    pub fn new(seed: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            generator: WorldGenerator::new(seed),
        }
    }

    pub fn get_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &Chunk {
        self.chunks
            .entry((chunk_x, chunk_z))
            .or_insert_from(|| self.generator.generate_chunk(chunk_x, chunk_z));
        self.chunks.get(&(chunk_x, chunk_z)).unwrap()
    }

    // Read-only get, doesn't generate
    pub fn get_chunk_ref(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        self.chunks.get(&(chunk_x, chunk_z))
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        let chunk_x = (x as f32 / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_z = (z as f32 / CHUNK_DEPTH as f32).floor() as i32;

        if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_z)) {
            let local_x = (x - chunk_x * CHUNK_WIDTH as i32) as usize;
            let local_y = y as usize;
            let local_z = (z - chunk_z * CHUNK_DEPTH as i32) as usize;

            if let Some(block) = chunk.get_block(local_x, local_y, local_z) {
                return Some(*block);
            }
        }
        None
    }

    // Only works if chunk is loaded
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        let chunk_x = (x as f32 / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_z = (z as f32 / CHUNK_DEPTH as f32).floor() as i32;

        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
            let local_x = (x - chunk_x * CHUNK_WIDTH as i32) as usize;
            let local_y = y as usize;
            let local_z = (z - chunk_z * CHUNK_DEPTH as i32) as usize;
            chunk.set_block(local_x, local_y, local_z, block.block_type);
        }
    }
}

// Helper for HashMap entry or_insert_from
trait EntryExt<'a, K, V> {
    fn or_insert_from<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V;
}

impl<'a, K, V> EntryExt<'a, K, V> for std::collections::hash_map::Entry<'a, K, V> {
    fn or_insert_from<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => entry.insert(default()),
        }
    }
}
