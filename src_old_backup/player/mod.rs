use crate::world::World;
// use crate::world::block::BlockType;
use glam::Vec3;

// Simple AABB
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}

pub fn check_collision(position: Vec3, world: &World) -> bool {
    // Player box (approx)
    let player_width = 0.6;
    let player_height = 1.8;

    let min = position - Vec3::new(player_width / 2.0, 0.0, player_width / 2.0);
    let max = position + Vec3::new(player_width / 2.0, player_height, player_width / 2.0);
    let player_box = AABB::new(min, max);

    // Check surrounding blocks
    let min_x = min.x.floor() as i32;
    let max_x = max.x.floor() as i32;
    let min_y = min.y.floor() as i32;
    let max_y = max.y.floor() as i32;
    let min_z = min.z.floor() as i32;
    let max_z = max.z.floor() as i32;

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if let Some(block) = world.get_block(x, y, z) {
                    if block.is_active() && !block.is_water() {
                        // Water is passable
                        let block_min = Vec3::new(x as f32, y as f32, z as f32);
                        let block_max = block_min + Vec3::ONE;
                        let block_box = AABB::new(block_min, block_max);

                        if player_box.intersects(&block_box) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[derive(Debug, Clone, Copy)]
pub struct RaycastResult {
    pub block_pos: (i32, i32, i32),
    pub face: (i32, i32, i32), // Normal of the face hit
    #[allow(dead_code)]
    pub dist: f32,
}

pub fn raycast(
    origin: Vec3,
    direction: Vec3,
    max_dist: f32,
    world: &World,
) -> Option<RaycastResult> {
    let mut t = 0.0;
    let step = 0.05; // Precision

    let dir = direction.normalize();

    let mut last_pos = (
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );

    while t < max_dist {
        t += step;
        let curr_pos = origin + dir * t;
        let bx = curr_pos.x.floor() as i32;
        let by = curr_pos.y.floor() as i32;
        let bz = curr_pos.z.floor() as i32;

        if let Some(block) = world.get_block(bx, by, bz) {
            if block.is_active() && !block.is_water() {
                // Hit
                // Determine face
                let dx = bx - last_pos.0;
                let dy = by - last_pos.1;
                let dz = bz - last_pos.2;

                // If we jumped more than one block or diagonal, this simple logic might fail slightly,
                // but for small steps it works. Better is DDA algorithm.
                // Assuming we came from last_pos which was empty.

                return Some(RaycastResult {
                    block_pos: (bx, by, bz),
                    face: (-dx, -dy, -dz), // Normal points back
                    dist: t,
                });
            }
        }
        last_pos = (bx, by, bz);
    }
    None
}
