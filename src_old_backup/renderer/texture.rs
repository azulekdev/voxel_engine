use gl;
use image::{Rgba, RgbaImage};
use std::ffi::c_void;

pub struct Texture {
    pub id: u32,
}

impl Texture {
    #[allow(dead_code)]
    pub fn new(_path: &str) -> Self {
        // Placeholder for loading from file
        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            // ... load image ...
        }
        Self { id: texture_id }
    }

    pub fn generate_atlas() -> Self {
        let mut texture_id = 0;
        let width = 256;
        let height = 256;
        let mut img = RgbaImage::new(width, height);

        // Define colors
        let dirt = Rgba([121, 85, 58, 255]);
        let grass_top = Rgba([65, 152, 10, 255]);
        let stone = Rgba([128, 128, 128, 255]);
        let water = Rgba([64, 164, 223, 200]); // Alpha handled in shader mostly, but store here too
        let oak_side = Rgba([102, 81, 60, 255]);
        let oak_top = Rgba([156, 126, 96, 255]);
        let leaves = Rgba([34, 139, 34, 255]); // Transparent-ish leaves?

        // Fill 16x16 slots (approx 16px each block if texture is 256x256)
        // Actually UVs in mesh are 0..1.
        // If we want texture atlas, we need to map UVs.
        // Let's make a grid.

        // Slot 0,0 (Top-left): Grass Block Top ??
        // We'll define UV mapping in mesh.rs later.

        // Let's just make it simple:
        // 0,0: Dirt
        fill_rect(&mut img, 0, 0, 16, 16, dirt);
        // 1,0: Stone
        fill_rect(&mut img, 16, 0, 32, 16, stone);
        // 2,0: Grass (Top)
        fill_rect(&mut img, 32, 0, 48, 16, grass_top);
        // 3,0: Water
        fill_rect(&mut img, 48, 0, 64, 16, water);
        // 4,0: Oak Side
        fill_rect(&mut img, 64, 0, 80, 16, oak_side);
        // 5,0: Oak Top
        fill_rect(&mut img, 80, 0, 96, 16, oak_top);
        // 6,0: Leaves
        fill_rect(&mut img, 96, 0, 112, 16, leaves);
        // 7,0: Grass Side (Dirt + Green top stripe)
        fill_rect(&mut img, 112, 0, 128, 16, dirt);
        // Add green stripe at top for grass side
        for x in 112..128 {
            for y in 0..3 {
                img.put_pixel(x, y, grass_top);
            }
        }

        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            // Texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32); // Pixel art look
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_raw().as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Self { id: texture_id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

fn fill_rect(img: &mut RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32, color: Rgba<u8>) {
    for x in x1..x2 {
        for y in y1..y2 {
            img.put_pixel(x, y, color);
        }
    }
}
