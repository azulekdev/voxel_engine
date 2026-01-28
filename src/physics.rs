use crate::world::World;
use glam::Vec3;

// Player bounding box
const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;

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

// PROPERLY WORKING collision detection
pub fn check_collision(pos: Vec3, world: &World) -> bool {
    // Camera position is at EYE HEIGHT (1.6 blocks above feet)
    const EYE_HEIGHT: f32 = 1.62;

    let player_box = AABB::new(
        Vec3::new(
            pos.x - PLAYER_WIDTH / 2.0,
            pos.y - EYE_HEIGHT, // Feet = eye - eye_height
            pos.z - PLAYER_WIDTH / 2.0,
        ),
        Vec3::new(
            pos.x + PLAYER_WIDTH / 2.0,
            pos.y + (PLAYER_HEIGHT - EYE_HEIGHT), // Head = eye + remaining height
            pos.z + PLAYER_WIDTH / 2.0,
        ),
    );

    // Check all blocks in player's bounding box
    let min_x = player_box.min.x.floor() as i32;
    let max_x = player_box.max.x.ceil() as i32;
    let min_y = player_box.min.y.floor() as i32;
    let max_y = player_box.max.y.ceil() as i32;
    let min_z = player_box.min.z.floor() as i32;
    let max_z = player_box.max.z.ceil() as i32;

    let mut checked = 0;
    let mut found_blocks = 0;
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                checked += 1;
                if let Some(block) = world.get_block(x, y, z) {
                    found_blocks += 1;
                    if block.is_solid() {
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

    if checked > 0 && found_blocks == 0 {
        println!(
            "WARNING: Checked {} positions but found 0 blocks! Pos: ({:.1}, {:.1}, {:.1})",
            checked, pos.x, pos.y, pos.z
        );
    }

    false
}

// Raycasting for block selection
pub struct RaycastHit {
    pub block_pos: (i32, i32, i32),
    pub face_normal: Vec3,
}

pub fn raycast(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    world: &World,
) -> Option<RaycastHit> {
    let step = 0.1;
    let mut current = origin;

    for _ in 0..(max_distance / step) as i32 {
        current += direction * step;

        let x = current.x.floor() as i32;
        let y = current.y.floor() as i32;
        let z = current.z.floor() as i32;

        if let Some(block) = world.get_block(x, y, z) {
            if block.is_solid() {
                // Determine which face was hit
                let block_center = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                let hit_offset = current - block_center;

                let face_normal = if hit_offset.x.abs() > hit_offset.y.abs()
                    && hit_offset.x.abs() > hit_offset.z.abs()
                {
                    Vec3::new(hit_offset.x.signum(), 0.0, 0.0)
                } else if hit_offset.y.abs() > hit_offset.z.abs() {
                    Vec3::new(0.0, hit_offset.y.signum(), 0.0)
                } else {
                    Vec3::new(0.0, 0.0, hit_offset.z.signum())
                };

                return Some(RaycastHit {
                    block_pos: (x, y, z),
                    face_normal,
                });
            }
        }
    }
    None
}
