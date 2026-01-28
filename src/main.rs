mod block;
mod camera;
mod chunk;
mod mesh;
mod physics;
mod shader;
mod terrain;
mod texture;
mod world;

use block::{Block, BlockType};
use camera::Camera;
use chunk::CHUNK_SIZE;
use glam::{Mat4, Vec3};
use glfw::{Action, Context, Key, MouseButton};
use mesh::generate_chunk_mesh;
use physics::{check_collision, raycast};
use shader::Shader;
use std::collections::HashMap;
use terrain::TerrainGenerator;
use texture::TextureAtlas;
use world::World;

const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;
const RENDER_DISTANCE: i32 = 5;

fn main() {
    // Initialize GLFW
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, events) = glfw
        .create_window(
            SCR_WIDTH,
            SCR_HEIGHT,
            "Voxel Engine by azul",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    // Load OpenGL
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        // DISABLE face culling temporarily to fix hollow blocks
        // gl::Enable(gl::CULL_FACE);
        // gl::CullFace(gl::BACK);
        // gl::FrontFace(gl::CCW);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // Uncapped FPS
    glfw.set_swap_interval(glfw::SwapInterval::None);

    // Create systems
    let shader = Shader::new("shaders/voxel.vert", "shaders/voxel.frag");
    let texture = TextureAtlas::new();
    let mut world = World::new();
    let terrain_gen = TerrainGenerator::new(12345);

    let mut meshes: HashMap<(i32, i32), mesh::Mesh> = HashMap::new();

    // Pass 1: Generate all chunks
    println!("Generating world (Pass 1/2: Chunks)...");
    for cx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for cz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk = terrain_gen.generate_chunk(cx, cz);
            world.chunks.insert((cx, cz), chunk);
        }
        glfw.poll_events(); // Keep window responsive
    }

    // Pass 2: Generate all meshes (now that neighbors exist)
    println!("Generating world (Pass 2/2: Meshes)...");
    for cx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for cz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            if let Some(chunk) = world.chunks.get(&(cx, cz)) {
                let mesh = generate_chunk_mesh(&world, chunk);
                meshes.insert((cx, cz), mesh);
            }
        }
        glfw.poll_events(); // Keep window responsive
        println!("Generated mesh row: cx={}", cx);
    }
    println!("World generation complete!");

    // Find spawn height (highest block at 0,0) - Spawn from the SKY!
    let mut spawn_y = 150.0;
    for y in (0..256).rev() {
        if let Some(block) = world.get_block(0, y, 0) {
            if block.is_solid() {
                spawn_y = y as f32 + 60.0; // Start 60 blocks above the ground
                break;
            }
        }
    }
    println!("Dropping in from Y={}", spawn_y);

    let mut camera = Camera::new(Vec3::new(0.0, spawn_y, 0.0));
    let mut velocity = Vec3::ZERO;
    let mut is_flying = false; // Start in walking mode
    let gravity = -20.0; // Negative for downward

    let mut is_paused = false;
    let mut last_frame = glfw.get_time() as f32;
    let mut delta_time;
    let mut last_x = SCR_WIDTH as f32 / 2.0;
    let mut last_y = SCR_HEIGHT as f32 / 2.0;
    let mut first_mouse = true;
    let mut last_space_time = 0.0;
    let speed = 10.0;
    let mut held_block = BlockType::Stone;

    // Main loop
    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;

        // Events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    is_paused = !is_paused;
                    if is_paused {
                        window.set_cursor_mode(glfw::CursorMode::Normal);
                    } else {
                        window.set_cursor_mode(glfw::CursorMode::Disabled);
                        first_mouse = true;
                    }
                }
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    // Double-tap to toggle flying
                    if current_frame - last_space_time < 0.3 {
                        is_flying = !is_flying;
                        println!("Flying mode: {}", if is_flying { "ON" } else { "OFF" });
                    }
                    last_space_time = current_frame;
                }
                glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) => {
                    held_block = BlockType::Dirt
                }
                glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) => {
                    held_block = BlockType::Stone
                }
                glfw::WindowEvent::Key(Key::Num3, _, Action::Press, _) => {
                    held_block = BlockType::Grass
                }
                glfw::WindowEvent::Key(Key::Num4, _, Action::Press, _) => {
                    held_block = BlockType::OakLog
                }
                glfw::WindowEvent::Key(Key::Num5, _, Action::Press, _) => {
                    held_block = BlockType::Leaves
                }
                glfw::WindowEvent::CursorPos(xpos, ypos) => {
                    if !is_paused {
                        if first_mouse {
                            last_x = xpos as f32;
                            last_y = ypos as f32;
                            first_mouse = false;
                        }
                        let xoffset = xpos as f32 - last_x;
                        let yoffset = last_y - ypos as f32;
                        last_x = xpos as f32;
                        last_y = ypos as f32;
                        camera.process_mouse(xoffset, yoffset);
                    }
                }
                glfw::WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                    if !is_paused {
                        // Break block
                        if let Some(hit) = raycast(camera.position, camera.front, 10.0, &world) {
                            world.set_block(
                                hit.block_pos.0,
                                hit.block_pos.1,
                                hit.block_pos.2,
                                Block::new(BlockType::Air),
                            );
                            // Regenerate chunk mesh
                            let chunk_x = hit.block_pos.0.div_euclid(CHUNK_SIZE as i32);
                            let chunk_z = hit.block_pos.2.div_euclid(CHUNK_SIZE as i32);
                            if let Some(chunk) = world.chunks.get(&(chunk_x, chunk_z)) {
                                let new_mesh = generate_chunk_mesh(&world, chunk);
                                meshes.insert((chunk_x, chunk_z), new_mesh);
                            }
                        }
                    }
                }
                glfw::WindowEvent::MouseButton(MouseButton::Button2, Action::Press, _) => {
                    if !is_paused {
                        // Place block
                        if let Some(hit) = raycast(camera.position, camera.front, 10.0, &world) {
                            let place_pos = (
                                hit.block_pos.0 + hit.face_normal.x as i32,
                                hit.block_pos.1 + hit.face_normal.y as i32,
                                hit.block_pos.2 + hit.face_normal.z as i32,
                            );
                            world.set_block(
                                place_pos.0,
                                place_pos.1,
                                place_pos.2,
                                Block::new(held_block),
                            );
                            // Regenerate chunk mesh
                            let chunk_x = place_pos.0.div_euclid(CHUNK_SIZE as i32);
                            let chunk_z = place_pos.2.div_euclid(CHUNK_SIZE as i32);
                            if let Some(chunk) = world.chunks.get(&(chunk_x, chunk_z)) {
                                let new_mesh = generate_chunk_mesh(&world, chunk);
                                meshes.insert((chunk_x, chunk_z), new_mesh);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if !is_paused {
            // Movement
            let mut move_dir = Vec3::ZERO;
            if window.get_key(Key::W) == Action::Press {
                move_dir += camera.front;
            }
            if window.get_key(Key::S) == Action::Press {
                move_dir -= camera.front;
            }
            if window.get_key(Key::A) == Action::Press {
                move_dir -= camera.right;
            }
            if window.get_key(Key::D) == Action::Press {
                move_dir += camera.right;
            }

            if is_flying {
                // Flying mode
                if window.get_key(Key::Space) == Action::Press {
                    move_dir += camera.world_up;
                }
                if window.get_key(Key::LeftShift) == Action::Press {
                    move_dir -= camera.world_up;
                }

                if move_dir.length_squared() > 0.0 {
                    move_dir = move_dir.normalize();
                }

                let displacement = move_dir * speed * delta_time;

                // Apply collision
                let mut next_pos = camera.position;
                next_pos.x += displacement.x;
                if !check_collision(next_pos, &world) {
                    camera.position.x += displacement.x;
                }

                next_pos = camera.position;
                next_pos.y += displacement.y;
                if !check_collision(next_pos, &world) {
                    camera.position.y += displacement.y;
                }

                next_pos = camera.position;
                next_pos.z += displacement.z;
                if !check_collision(next_pos, &world) {
                    camera.position.z += displacement.z;
                }

                velocity = Vec3::ZERO;
            } else {
                // Walking mode
                move_dir.y = 0.0;
                if move_dir.length_squared() > 0.0 {
                    move_dir = move_dir.normalize();
                }

                let target_vel = move_dir * speed;
                velocity.x = target_vel.x;
                velocity.z = target_vel.z;

                // Gravity
                velocity.y += gravity * delta_time;

                // Jumping
                if window.get_key(Key::Space) == Action::Press {
                    // Check if on ground
                    let mut ground_check = camera.position;
                    ground_check.y -= 0.1;
                    if check_collision(ground_check, &world) {
                        velocity.y = 8.0; // Jump force
                    }
                }

                let displacement = velocity * delta_time;

                // Collision
                let mut next_pos = camera.position;
                next_pos.x += displacement.x;
                if check_collision(next_pos, &world) {
                    velocity.x = 0.0;
                } else {
                    camera.position.x += displacement.x;
                }

                next_pos = camera.position;
                next_pos.z += displacement.z;
                if check_collision(next_pos, &world) {
                    velocity.z = 0.0;
                } else {
                    camera.position.z += displacement.z;
                }

                next_pos = camera.position;
                next_pos.y += displacement.y;
                if check_collision(next_pos, &world) {
                    velocity.y = 0.0;
                } else {
                    camera.position.y += displacement.y;
                }
            }
        }

        // Update window title
        let fps = 1.0 / delta_time;
        let title = format!(
            "Voxel Engine by azul | FPS: {:.0} | Pos: ({:.1}, {:.1}, {:.1}) | Block: {:?}",
            fps, camera.position.x, camera.position.y, camera.position.z, held_block
        );
        window.set_title(&title);

        // Render
        unsafe {
            gl::ClearColor(0.53, 0.81, 0.92, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        shader.use_program();
        texture.bind();

        let projection = Mat4::perspective_rh_gl(
            80.0_f32.to_radians(),
            SCR_WIDTH as f32 / SCR_HEIGHT as f32,
            0.1,
            500.0,
        );
        let view = camera.get_view_matrix();
        shader.set_mat4("projection", &projection);
        shader.set_mat4("view", &view);
        shader.set_vec3("viewPos", &camera.position);
        shader.set_vec3("lightDir", &Vec3::new(0.5, -1.0, 0.5));
        shader.set_int("blockTexture", 0);
        shader.set_bool("isWater", false);

        // Render chunks
        for ((cx, cz), mesh) in &meshes {
            let pos = Vec3::new(
                (*cx * CHUNK_SIZE as i32) as f32,
                0.0,
                (*cz * CHUNK_SIZE as i32) as f32,
            );
            let model = Mat4::from_translation(pos);
            shader.set_mat4("model", &model);
            mesh.draw();
        }

        // Crosshair
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(SCR_WIDTH as i32 / 2 - 1, SCR_HEIGHT as i32 / 2 - 1, 3, 3);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Disable(gl::SCISSOR_TEST);
            gl::Enable(gl::DEPTH_TEST);
        }

        window.swap_buffers();
    }
}
