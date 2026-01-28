use crate::block::BlockType;
use crate::chunk::{CHUNK_HEIGHT, CHUNK_SIZE, Chunk};
use crate::texture::TextureAtlas;
use crate::world::World;

pub struct Mesh {
    vao: u32,
    pub vertex_count: i32,
}

impl Mesh {
    pub fn new(vertices: &[f32]) -> Self {
        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Position (3 floats)
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                9 * std::mem::size_of::<f32>() as i32,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // TexCoord (2 floats)
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                9 * std::mem::size_of::<f32>() as i32,
                (3 * std::mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            // Normal (3 floats)
            gl::VertexAttribPointer(
                2,
                3,
                gl::FLOAT,
                gl::FALSE,
                9 * std::mem::size_of::<f32>() as i32,
                (5 * std::mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(2);

            // AO (1 float)
            gl::VertexAttribPointer(
                3,
                1,
                gl::FLOAT,
                gl::FALSE,
                9 * std::mem::size_of::<f32>() as i32,
                (8 * std::mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(3);

            gl::BindVertexArray(0);
        }

        Self {
            vao,
            vertex_count: (vertices.len() / 9) as i32,
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
            gl::BindVertexArray(0);
        }
    }
}

// Generate mesh for a chunk with ambient occlusion
pub fn generate_chunk_mesh(world: &World, chunk: &Chunk) -> Mesh {
    let mut vertices = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                let block = chunk.get_block(x, y, z).unwrap();
                if block.block_type == BlockType::Air {
                    continue;
                }

                // Check each face
                for face in 0..6 {
                    if should_render_face(
                        world,
                        chunk,
                        x as i32,
                        y as i32,
                        z as i32,
                        face,
                        &block.block_type,
                    ) {
                        add_face(
                            &mut vertices,
                            x as f32,
                            y as f32,
                            z as f32,
                            face,
                            &block.block_type,
                            chunk,
                            x as i32,
                            y as i32,
                            z as i32,
                        );
                    }
                }
            }
        }
    }

    Mesh::new(&vertices)
}

fn should_render_face(
    world: &World,
    chunk: &Chunk,
    x: i32,
    y: i32,
    z: i32,
    face: usize,
    block_type: &BlockType,
) -> bool {
    let (dx, dy, dz) = match face {
        0 => (-1, 0, 0), // Left
        1 => (1, 0, 0),  // Right
        2 => (0, -1, 0), // Bottom
        3 => (0, 1, 0),  // Top
        4 => (0, 0, -1), // Back
        5 => (0, 0, 1),  // Front
        _ => (0, 0, 0),
    };

    let nx = x + dx;
    let ny = y + dy;
    let nz = z + dz;

    // OPTIMIZATION: Only use global world lookup if at chunk boundary
    let n_block = if nx < 0
        || nx >= CHUNK_SIZE as i32
        || ny < 0
        || ny >= CHUNK_HEIGHT as i32
        || nz < 0
        || nz >= CHUNK_SIZE as i32
    {
        let gx = chunk.x * CHUNK_SIZE as i32 + nx;
        let gy = ny;
        let gz = chunk.z * CHUNK_SIZE as i32 + nz;

        let Some(block) = world.get_block(gx, gy, gz) else {
            return true;
        };
        block
    } else {
        *chunk
            .get_block(nx as usize, ny as usize, nz as usize)
            .unwrap()
    };

    // Water renders through transparent blocks
    if *block_type == BlockType::Water {
        // Only cull against other water blocks to make it look connected
        if n_block.block_type == BlockType::Water {
            return false;
        }
        return n_block.block_type == BlockType::Air || n_block.block_type == BlockType::Leaves;
    }

    // Leaves should not cull against other leaves to look "fancy" and not hollow
    if *block_type == BlockType::Leaves {
        if n_block.block_type == BlockType::Leaves {
            return true; // Render internal leaf faces
        }
        return n_block.is_transparent();
    }

    n_block.is_transparent()
}

// Calculate ambient occlusion for a vertex
fn calculate_ao(chunk: &Chunk, x: i32, y: i32, z: i32, face: usize, corner: usize) -> f32 {
    // Get the 3 neighbors for this corner
    let (side1, side2, corner_block) = get_ao_neighbors(face, corner);

    let s1 = is_solid_at(chunk, x + side1.0, y + side1.1, z + side1.2);
    let s2 = is_solid_at(chunk, x + side2.0, y + side2.1, z + side2.2);
    let c = is_solid_at(
        chunk,
        x + corner_block.0,
        y + corner_block.1,
        z + corner_block.2,
    );

    // AO calculation
    if s1 && s2 {
        0.25 // Darkest
    } else {
        1.0 - ((s1 as i32 + s2 as i32 + c as i32) as f32 * 0.25)
    }
}

fn is_solid_at(chunk: &Chunk, x: i32, y: i32, z: i32) -> bool {
    if x < 0
        || x >= CHUNK_SIZE as i32
        || y < 0
        || y >= CHUNK_HEIGHT as i32
        || z < 0
        || z >= CHUNK_SIZE as i32
    {
        return false;
    }
    let block = chunk.get_block(x as usize, y as usize, z as usize).unwrap();
    block.is_solid()
}

fn get_ao_neighbors(
    face: usize,
    corner: usize,
) -> ((i32, i32, i32), (i32, i32, i32), (i32, i32, i32)) {
    // Returns (side1, side2, corner) offsets for AO calculation
    match (face, corner) {
        // Top face (y+)
        (3, 0) => ((-1, 1, 0), (0, 1, -1), (-1, 1, -1)),
        (3, 1) => ((1, 1, 0), (0, 1, -1), (1, 1, -1)),
        (3, 2) => ((1, 1, 0), (0, 1, 1), (1, 1, 1)),
        (3, 3) => ((-1, 1, 0), (0, 1, 1), (-1, 1, 1)),
        // Add other faces as needed...
        _ => ((0, 0, 0), (0, 0, 0), (0, 0, 0)),
    }
}

fn add_face(
    vertices: &mut Vec<f32>,
    x: f32,
    y: f32,
    z: f32,
    face: usize,
    block_type: &BlockType,
    chunk: &Chunk,
    bx: i32,
    by: i32,
    bz: i32,
) {
    let (positions, normal) = get_face_data(face);
    let (u_min, v_min, u_max, v_max) = TextureAtlas::get_uv(block_type, face);

    // Calculate AO for each corner
    let ao = [
        calculate_ao(chunk, bx, by, bz, face, 0),
        calculate_ao(chunk, bx, by, bz, face, 1),
        calculate_ao(chunk, bx, by, bz, face, 2),
        calculate_ao(chunk, bx, by, bz, face, 3),
    ];

    // Two triangles per face
    let indices = [0, 1, 2, 0, 2, 3];
    let uvs = [
        (u_min, v_max),
        (u_max, v_max),
        (u_max, v_min),
        (u_min, v_min),
    ];

    for &i in &indices {
        let mut pos = positions[i];
        let uv = uvs[i];

        // Half-height water
        if *block_type == BlockType::Water {
            // If it's the top face (face 3), or the top vertices of side faces
            if face == 3 {
                pos.1 = 0.8; // Water is 80% height
            } else if face != 2 {
                // Not the bottom face
                // For side faces, lower the top vertices
                if pos.1 > 0.0 {
                    pos.1 = 0.8;
                }
            }
        }

        // Position
        vertices.push(x + pos.0);
        vertices.push(y + pos.1);
        vertices.push(z + pos.2);

        // TexCoord
        vertices.push(uv.0);
        vertices.push(uv.1);

        // Normal
        vertices.push(normal.0);
        vertices.push(normal.1);
        vertices.push(normal.2);

        // AO
        vertices.push(ao[i]);
    }
}

fn get_face_data(face: usize) -> ([(f32, f32, f32); 4], (f32, f32, f32)) {
    match face {
        0 => (
            [
                (0.0, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                (0.0, 1.0, 1.0),
                (0.0, 0.0, 1.0),
            ],
            (-1.0, 0.0, 0.0),
        ), // Left
        1 => (
            [
                (1.0, 0.0, 1.0),
                (1.0, 1.0, 1.0),
                (1.0, 1.0, 0.0),
                (1.0, 0.0, 0.0),
            ],
            (1.0, 0.0, 0.0),
        ), // Right
        2 => (
            [
                (0.0, 0.0, 0.0),
                (1.0, 0.0, 0.0),
                (1.0, 0.0, 1.0),
                (0.0, 0.0, 1.0),
            ],
            (0.0, -1.0, 0.0),
        ), // Bottom
        3 => (
            [
                (0.0, 1.0, 1.0),
                (1.0, 1.0, 1.0),
                (1.0, 1.0, 0.0),
                (0.0, 1.0, 0.0),
            ],
            (0.0, 1.0, 0.0),
        ), // Top
        4 => (
            [
                (1.0, 0.0, 0.0),
                (1.0, 1.0, 0.0),
                (0.0, 1.0, 0.0),
                (0.0, 0.0, 0.0),
            ],
            (0.0, 0.0, -1.0),
        ), // Back
        5 => (
            [
                (0.0, 0.0, 1.0),
                (0.0, 1.0, 1.0),
                (1.0, 1.0, 1.0),
                (1.0, 0.0, 1.0),
            ],
            (0.0, 0.0, 1.0),
        ), // Front
        _ => ([(0.0, 0.0, 0.0); 4], (0.0, 0.0, 0.0)),
    }
}
