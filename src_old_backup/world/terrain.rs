use super::block::BlockType;
use super::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use glam::Vec3;
use noise::{NoiseFn, Perlin};
use rand::Rng;

pub struct WorldGenerator {
    perlin: Perlin,
    seed: u32,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            seed,
        }
    }

    pub fn generate_chunk(&self, chunk_x: i32, chunk_z: i32) -> Chunk {
        let mut chunk = Chunk::new(Vec3::new(
            (chunk_x * CHUNK_WIDTH as i32) as f32,
            0.0,
            (chunk_z * CHUNK_DEPTH as i32) as f32,
        ));

        let water_level = 64;
        let mountain_level = 100;

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let world_x = (chunk_x * CHUNK_WIDTH as i32) + x as i32;
                let world_z = (chunk_z * CHUNK_DEPTH as i32) + z as i32;

                // Height noise (Scale: 0.01)
                let noise_val = self
                    .perlin
                    .get([world_x as f64 * 0.01, world_z as f64 * 0.01]);
                // Map -1..1 to reasonable height (e.g., 40..150)
                let height = ((noise_val + 1.0) * 0.5 * 100.0 + 40.0) as usize;

                for y in 0..CHUNK_HEIGHT {
                    if y == 0 {
                        chunk.set_block(x, y, z, BlockType::Stone); // Bedrock
                    } else if y < height {
                        if y > mountain_level {
                            chunk.set_block(x, y, z, BlockType::Stone);
                        } else if y == height - 1 {
                            // Top layer
                            if y < water_level + 2 {
                                // Beach
                                chunk.set_block(x, y, z, BlockType::Dirt); // Sand maybe? Using Dirt for now
                            } else {
                                chunk.set_block(x, y, z, BlockType::Grass);
                            }
                        } else if y > height - 5 {
                            chunk.set_block(x, y, z, BlockType::Dirt);
                        } else {
                            chunk.set_block(x, y, z, BlockType::Stone);
                        }
                    } else if y <= water_level {
                        chunk.set_block(x, y, z, BlockType::Water);
                    }
                }

                // Trees
                if height > water_level && height < mountain_level {
                    // Simple random tree placement
                    // Use a consistent hash or noise for trees to be deterministic
                    if self.should_place_tree(world_x, world_z) {
                        self.generate_tree(&mut chunk, x, height, z);
                    }
                }
            }
        }

        chunk
    }

    fn should_place_tree(&self, x: i32, z: i32) -> bool {
        // Pseudo-random based on position
        let mut rng = rand::rngs::StdRng::seed_from_u64(
            (x as u64).wrapping_mul(73856093)
                ^ (z as u64).wrapping_mul(19349663)
                ^ (self.seed as u64),
        );
        use rand::SeedableRng;
        rng.gen_bool(0.02) // 2% chance
    }

    fn generate_tree(&self, chunk: &mut Chunk, x: usize, base_y: usize, z: usize) {
        let height = 5;
        // Trunk
        for i in 0..height {
            if base_y + i < CHUNK_HEIGHT {
                chunk.set_block(x, base_y + i, z, BlockType::OakLog);
            }
        }

        // Leaves
        let leave_start = base_y + height - 2;
        for ly in leave_start..(base_y + height + 2) {
            for lx in (x as i32 - 2)..=(x as i32 + 2) {
                for lz in (z as i32 - 2)..=(z as i32 + 2) {
                    if lx >= 0 && lx < CHUNK_WIDTH as i32 && lz >= 0 && lz < CHUNK_DEPTH as i32 {
                        if ly < CHUNK_HEIGHT {
                            // Check bounds
                            // Simple sphere approximation or just box
                            if (lx as f32 - x as f32).abs() + (lz as f32 - z as f32).abs() <= 3.0 {
                                // Don't replace existing logs
                                if chunk
                                    .get_block(lx as usize, ly, lz as usize)
                                    .map(|b| !b.is_active())
                                    .unwrap_or(true)
                                {
                                    chunk.set_block(
                                        lx as usize,
                                        ly,
                                        lz as usize,
                                        BlockType::Leaves,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
