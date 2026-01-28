use crate::world::block::BlockType;
use crate::world::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use gl::types::*;
use std::mem;
use std::ptr;

pub struct Mesh {
    vao: u32,
    #[allow(dead_code)]
    vbo: u32,
    pub index_count: i32,
}

impl Mesh {
    pub fn new(vertices: &[f32]) -> Self {
        let mut vao = 0;
        let mut vbo = 0;
        let index_count = (vertices.len() / 8) as i32;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<f32>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Pos
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
            // Tex
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                (3 * mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(1);
            // Norm
            gl::VertexAttribPointer(
                2,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                (5 * mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(2);
        }

        Self {
            vao,
            vbo,
            index_count,
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.index_count);
        }
    }
}

pub fn generate_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut vertices: Vec<f32> = Vec::new();

    for x in 0..CHUNK_WIDTH {
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_DEPTH {
                let block = chunk.blocks[x][y][z];
                if !block.is_active() {
                    continue;
                }

                let pos = [x as f32, y as f32, z as f32];
                let is_water = block.is_water();

                // Front (Z+)
                if should_render_face(chunk, x as i32, y as i32, z as i32 + 1, is_water) {
                    add_face(&mut vertices, pos, [0.0, 0.0, 1.0], block.block_type, 0);
                }
                // Back (Z-)
                if should_render_face(chunk, x as i32, y as i32, z as i32 - 1, is_water) {
                    add_face(&mut vertices, pos, [0.0, 0.0, -1.0], block.block_type, 1);
                }
                // Left (X-)
                if should_render_face(chunk, x as i32 - 1, y as i32, z as i32, is_water) {
                    add_face(&mut vertices, pos, [-1.0, 0.0, 0.0], block.block_type, 2);
                }
                // Right (X+)
                if should_render_face(chunk, x as i32 + 1, y as i32, z as i32, is_water) {
                    add_face(&mut vertices, pos, [1.0, 0.0, 0.0], block.block_type, 3);
                }
                // Top (Y+)
                if should_render_face(chunk, x as i32, y as i32 + 1, z as i32, is_water) {
                    add_face(&mut vertices, pos, [0.0, 1.0, 0.0], block.block_type, 4);
                }
                // Bottom (Y-)
                if should_render_face(chunk, x as i32, y as i32 - 1, z as i32, is_water) {
                    add_face(&mut vertices, pos, [0.0, -1.0, 0.0], block.block_type, 5);
                }
            }
        }
    }

    Mesh::new(&vertices)
}

fn should_render_face(chunk: &Chunk, x: i32, y: i32, z: i32, self_is_water: bool) -> bool {
    if x < 0
        || x >= CHUNK_WIDTH as i32
        || y < 0
        || y >= CHUNK_HEIGHT as i32
        || z < 0
        || z >= CHUNK_DEPTH as i32
    {
        return true;
    }
    let neighbor = &chunk.blocks[x as usize][y as usize][z as usize];
    if self_is_water {
        return !neighbor.is_active() || neighbor.block_type == BlockType::Leaves;
    }
    neighbor.is_transparent()
}

// Helper to get UVs
// Atlas is 256x256, 16x16 tiles.
// Returns (u_min, v_min, u_max, v_max)
fn get_uv_coords(block_type: BlockType, face_idx: usize) -> (f32, f32, f32, f32) {
    let (col, row) = match block_type {
        BlockType::Dirt => (0, 0),
        BlockType::Stone => (1, 0),
        BlockType::Grass => {
            if face_idx == 4 {
                (2, 0)
            }
            // Top
            else if face_idx == 5 {
                (0, 0)
            }
            // Bottom (Dirt)
            else {
                (7, 0)
            } // Side
        }
        BlockType::Water => (3, 0),
        BlockType::OakLog => {
            if face_idx == 4 || face_idx == 5 {
                (5, 0)
            }
            // Top/Bottom
            else {
                (4, 0)
            } // Side
        }
        BlockType::Leaves => (6, 0),
        _ => (0, 0),
    };

    let size = 16.0 / 256.0;
    // In OpenGL UVs, (0,0) is bottom-left usually, but image crate loads top-left as (0,0) data.
    // However, we upload as RGBA.
    // If we want (0,0) to be top-left of image, V coordinate needs to be flipped if GL expects bottom-left.
    // But let's assume standard UV: (0,0) bottom-left.
    // Image crate data: Row 0 is at top.
    // So Row 0 needs to be at V = 1.0 (top) if we upload directly?
    // Actually, `glTexImage2D` reads pixels from bottom to top? No, usually generic pointer is row by row.
    // Standard GL convention: (0,0) is bottom-left.
    // RgbaImage layout: (0,0) is top-left pixel.
    // So row 0 is V=1.0? or V=0?
    // Let's assume standard mapping: U=x/W, V=y/H.
    // If we use standard UVs (0 at bottom, 1 at top), and we upload image where row 0 is top...
    // then row 0 corresponds to V=1 (approx).
    // Let's use V from 1.0 downwards.

    // Row 0 is top (V=1.0 -> V=1.0-size).
    // Row 1 is below.

    let u_min = col as f32 * size;
    let u_max = (col as f32 + 1.0) * size;
    let v_max = 1.0 - (row as f32 * size); // Top of the tile
    let v_min = 1.0 - ((row as f32 + 1.0) * size); // Bottom of the tile

    (u_min, v_min, u_max, v_max)
}

fn add_face(
    vertices: &mut Vec<f32>,
    pos: [f32; 3],
    normal: [f32; 3],
    block_type: BlockType,
    face_idx: usize,
) {
    let x = pos[0];
    let y = pos[1];
    let z = pos[2];

    let (u_min, v_min, u_max, v_max) = get_uv_coords(block_type, face_idx);

    // Top Face (0, 1, 0)
    if normal[1] > 0.0 {
        vertices.extend_from_slice(&[
            x,
            y + 1.0,
            z + 1.0,
            u_min,
            v_min,
            0.0,
            1.0,
            0.0, // BL
            x + 1.0,
            y + 1.0,
            z + 1.0,
            u_max,
            v_min,
            0.0,
            1.0,
            0.0, // BR
            x + 1.0,
            y + 1.0,
            z,
            u_max,
            v_max,
            0.0,
            1.0,
            0.0, // TR
            x,
            y + 1.0,
            z + 1.0,
            u_min,
            v_min,
            0.0,
            1.0,
            0.0,
            x + 1.0,
            y + 1.0,
            z,
            u_max,
            v_max,
            0.0,
            1.0,
            0.0,
            x,
            y + 1.0,
            z,
            u_min,
            v_max,
            0.0,
            1.0,
            0.0, // TL
        ]);
        return;
    }
    // Bottom Face (0, -1, 0)
    if normal[1] < 0.0 {
        vertices.extend_from_slice(&[
            x,
            y,
            z,
            u_min,
            v_min,
            0.0,
            -1.0,
            0.0,
            x + 1.0,
            y,
            z,
            u_max,
            v_min,
            0.0,
            -1.0,
            0.0,
            x + 1.0,
            y,
            z + 1.0,
            u_max,
            v_max,
            0.0,
            -1.0,
            0.0,
            x,
            y,
            z,
            u_min,
            v_min,
            0.0,
            -1.0,
            0.0,
            x + 1.0,
            y,
            z + 1.0,
            u_max,
            v_max,
            0.0,
            -1.0,
            0.0,
            x,
            y,
            z + 1.0,
            u_min,
            v_max,
            0.0,
            -1.0,
            0.0,
        ]);
        return;
    }

    // Front (0, 0, 1)
    if normal[2] > 0.0 {
        vertices.extend_from_slice(&[
            x,
            y,
            z + 1.0,
            u_min,
            v_min,
            0.0,
            0.0,
            1.0,
            x + 1.0,
            y,
            z + 1.0,
            u_max,
            v_min,
            0.0,
            0.0,
            1.0,
            x + 1.0,
            y + 1.0,
            z + 1.0,
            u_max,
            v_max,
            0.0,
            0.0,
            1.0,
            x,
            y,
            z + 1.0,
            u_min,
            v_min,
            0.0,
            0.0,
            1.0,
            x + 1.0,
            y + 1.0,
            z + 1.0,
            u_max,
            v_max,
            0.0,
            0.0,
            1.0,
            x,
            y + 1.0,
            z + 1.0,
            u_min,
            v_max,
            0.0,
            0.0,
            1.0,
        ]);
        return;
    }

    // Back (0, 0, -1)
    if normal[2] < 0.0 {
        vertices.extend_from_slice(&[
            x + 1.0,
            y,
            z,
            u_min,
            v_min,
            0.0,
            0.0,
            -1.0,
            x,
            y,
            z,
            u_max,
            v_min,
            0.0,
            0.0,
            -1.0,
            x,
            y + 1.0,
            z,
            u_max,
            v_max,
            0.0,
            0.0,
            -1.0,
            x + 1.0,
            y,
            z,
            u_min,
            v_min,
            0.0,
            0.0,
            -1.0,
            x,
            y + 1.0,
            z,
            u_max,
            v_max,
            0.0,
            0.0,
            -1.0,
            x + 1.0,
            y + 1.0,
            z,
            u_min,
            v_max,
            0.0,
            0.0,
            -1.0,
        ]);
        return;
    }

    // Right (1, 0, 0)
    if normal[0] > 0.0 {
        vertices.extend_from_slice(&[
            x + 1.0,
            y,
            z + 1.0,
            u_min,
            v_min,
            1.0,
            0.0,
            0.0,
            x + 1.0,
            y,
            z,
            u_max,
            v_min,
            1.0,
            0.0,
            0.0,
            x + 1.0,
            y + 1.0,
            z,
            u_max,
            v_max,
            1.0,
            0.0,
            0.0,
            x + 1.0,
            y,
            z + 1.0,
            u_min,
            v_min,
            1.0,
            0.0,
            0.0,
            x + 1.0,
            y + 1.0,
            z,
            u_max,
            v_max,
            1.0,
            0.0,
            0.0,
            x + 1.0,
            y + 1.0,
            z + 1.0,
            u_min,
            v_max,
            1.0,
            0.0,
            0.0,
        ]);
        return;
    }

    // Left (-1, 0, 0)
    if normal[0] < 0.0 {
        vertices.extend_from_slice(&[
            x,
            y,
            z,
            u_min,
            v_min,
            -1.0,
            0.0,
            0.0,
            x,
            y,
            z + 1.0,
            u_max,
            v_min,
            -1.0,
            0.0,
            0.0,
            x,
            y + 1.0,
            z + 1.0,
            u_max,
            v_max,
            -1.0,
            0.0,
            0.0,
            x,
            y,
            z,
            u_min,
            v_min,
            -1.0,
            0.0,
            0.0,
            x,
            y + 1.0,
            z + 1.0,
            u_max,
            v_max,
            -1.0,
            0.0,
            0.0,
            x,
            y + 1.0,
            z,
            u_min,
            v_max,
            -1.0,
            0.0,
            0.0,
        ]);
        return;
    }
}
