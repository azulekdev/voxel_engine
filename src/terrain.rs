use crate::block::BlockType;
use crate::chunk::{CHUNK_HEIGHT, CHUNK_SIZE, Chunk};
use noise::{NoiseFn, Perlin};

pub struct TerrainGenerator {
    noise: Perlin,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
        }
    }

    pub fn generate_chunk(&self, chunk_x: i32, chunk_z: i32) -> Chunk {
        let mut chunk = Chunk::new(chunk_x, chunk_z);

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = chunk_x * CHUNK_SIZE as i32 + x as i32;
                let world_z = chunk_z * CHUNK_SIZE as i32 + z as i32;

                // Multi-octave noise for realistic terrain
                let height = self.get_height(world_x, world_z);

                for y in 0..CHUNK_HEIGHT {
                    let block_type = if y < 60 {
                        // Water level
                        if y < height {
                            if y < height - 4 {
                                BlockType::Stone
                            } else if y < height - 1 {
                                BlockType::Dirt
                            } else {
                                BlockType::Grass
                            }
                        } else {
                            BlockType::Water
                        }
                    } else {
                        // Above water
                        if y < height {
                            if y < height - 4 {
                                BlockType::Stone
                            } else if y < height - 1 {
                                BlockType::Dirt
                            } else {
                                BlockType::Grass
                            }
                        } else {
                            BlockType::Air
                        }
                    };

                    chunk.set_block(x, y, z, block_type);
                }

                // Add trees
                if height > 62 && self.should_place_tree(world_x, world_z) {
                    self.add_tree(&mut chunk, x, height, z);
                }
            }
        }

        chunk
    }

    fn get_height(&self, x: i32, y: i32) -> usize {
        let scale = 0.01;
        let x_scaled = x as f64 * scale;
        let y_scaled = y as f64 * scale;

        // Multiple octaves for varied terrain
        let noise1 = self.noise.get([x_scaled, y_scaled]) * 30.0;
        let noise2 = self.noise.get([x_scaled * 2.0, y_scaled * 2.0]) * 15.0;
        let noise3 = self.noise.get([x_scaled * 4.0, y_scaled * 4.0]) * 7.0;

        let height = 70.0 + noise1 + noise2 + noise3;
        height.max(1.0).min(CHUNK_HEIGHT as f64 - 1.0) as usize
    }

    fn should_place_tree(&self, x: i32, z: i32) -> bool {
        let tree_noise = self.noise.get([x as f64 * 0.1, z as f64 * 0.1]);
        tree_noise > 0.7
    }

    fn add_tree(&self, chunk: &mut Chunk, x: usize, base_y: usize, z: usize) {
        let trunk_height = 5;

        // Trunk
        for y in 0..trunk_height {
            if base_y + y < CHUNK_HEIGHT {
                chunk.set_block(x, base_y + y, z, BlockType::OakLog);
            }
        }

        // Leaves
        let leaf_y = base_y + trunk_height;
        for dx in -2..=2_i32 {
            for dz in -2..=2_i32 {
                for dy in 0..=2 {
                    if dx.abs() + dz.abs() + dy <= 3 {
                        let lx = x as i32 + dx;
                        let lz = z as i32 + dz;
                        let ly = leaf_y + dy as usize;

                        if lx >= 0
                            && lx < CHUNK_SIZE as i32
                            && lz >= 0
                            && lz < CHUNK_SIZE as i32
                            && ly < CHUNK_HEIGHT
                        {
                            chunk.set_block(lx as usize, ly, lz as usize, BlockType::Leaves);
                        }
                    }
                }
            }
        }
    }
}
