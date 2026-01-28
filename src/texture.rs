use image::{Rgba, RgbaImage};
use std::ffi::c_void;

pub struct TextureAtlas {
    pub id: u32,
}

impl TextureAtlas {
    pub fn new() -> Self {
        let mut texture_id = 0;
        let size = 256;
        let mut img = RgbaImage::new(size, size);

        // Fill with base color (not black!)
        for x in 0..size {
            for y in 0..size {
                img.put_pixel(x, y, Rgba([128, 128, 128, 255]));
            }
        }

        // Define SOLID block colors (16x16 tiles) - NO PATTERNS
        let tile_size = 16;

        // Tile 0: Dirt (solid brown)
        Self::fill_tile(&mut img, 0, 0, tile_size, Rgba([139, 90, 43, 255]));

        // Tile 1: Stone (solid gray)
        Self::fill_tile(&mut img, 1, 0, tile_size, Rgba([120, 120, 120, 255]));

        // Tile 2: Grass Top (solid vibrant green)
        Self::fill_tile(&mut img, 2, 0, tile_size, Rgba([76, 187, 23, 255]));

        // Tile 3: Grass Side (solid brown - will be same as dirt for now)
        Self::fill_tile(&mut img, 3, 0, tile_size, Rgba([139, 90, 43, 255]));

        // Tile 4: Water (darker cyan for visibility)
        Self::fill_tile(&mut img, 4, 0, tile_size, Rgba([30, 100, 180, 128])); // Semi-transparent

        // Tile 5: Oak Log Side (solid brown)
        Self::fill_tile(&mut img, 5, 0, tile_size, Rgba([102, 81, 60, 255]));

        // Tile 6: Oak Log Top (lighter brown)
        Self::fill_tile(&mut img, 6, 0, tile_size, Rgba([156, 126, 96, 255]));

        // Tile 7: Leaves (solid forest green)
        Self::fill_tile(&mut img, 7, 0, tile_size, Rgba([48, 168, 48, 255]));

        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                size as i32,
                size as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_raw().as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Self { id: texture_id }
    }

    fn fill_tile(img: &mut RgbaImage, tile_x: u32, tile_y: u32, tile_size: u32, color: Rgba<u8>) {
        for x in 0..tile_size {
            for y in 0..tile_size {
                let px = tile_x * tile_size + x;
                let py = tile_y * tile_size + y;
                img.put_pixel(px, py, color);
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    // Get UV coordinates for a block face
    pub fn get_uv(block_type: &crate::block::BlockType, face: usize) -> (f32, f32, f32, f32) {
        let tile_size = 16.0 / 256.0;

        let tile_index = match block_type {
            crate::block::BlockType::Dirt => 0,
            crate::block::BlockType::Stone => 1,
            crate::block::BlockType::Grass => {
                if face == 3 {
                    2
                }
                // Top (Y+)
                else if face == 2 {
                    0
                }
                // Bottom (dirt)
                else {
                    3
                } // Sides
            }
            crate::block::BlockType::Water => 4,
            crate::block::BlockType::OakLog => {
                if face == 4 || face == 5 {
                    6
                }
                // Top/bottom
                else {
                    5
                } // Sides
            }
            crate::block::BlockType::Leaves => 7,
            _ => 0,
        };

        let u_min = (tile_index as f32) * tile_size;
        let u_max = u_min + tile_size;
        let v_min = 0.0;
        let v_max = tile_size;

        (u_min, v_min, u_max, v_max)
    }
}
