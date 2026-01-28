use glam::{Mat4, Vec3};
use glfw::{Action, Context, Key, MouseButton};
use rand::Rng;
use std::collections::HashMap;
use std::io::{self, Write};

mod player;
mod renderer;
mod test_cube;
mod world;

use player::{check_collision, raycast};
use renderer::camera::Camera;
use renderer::mesh::{Mesh, generate_chunk_mesh};
use renderer::shader::Shader;
use renderer::texture::Texture;
use world::World;
use world::block::{Block, BlockType};
use world::chunk::CHUNK_WIDTH;

const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

fn main() {
    // 1. Seed Input
    print!("Enter seed (leave empty for random): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let seed_str = input.trim();
    let seed: u32 = if seed_str.is_empty() {
        let mut rng = rand::thread_rng();
        rng.r#gen()
    } else {
        match seed_str.parse() {
            Ok(s) => s,
            Err(_) => 42,
        }
    };
    println!("Starting engine with seed: {}", seed);

    // 2. Init GLFW
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, events) = glfw
        .create_window(
            SCR_WIDTH,
            SCR_HEIGHT,
            "Voxel Engine",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE); // Re-enable face culling
        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CCW); // Counter-clockwise winding
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // Uncapped FPS
    glfw.set_swap_interval(glfw::SwapInterval::None);

    let shader = Shader::new("shaders/voxel.vert", "shaders/voxel.frag");
    let texture = Texture::generate_atlas();

    // Initial Spawn Position (High enough to not fall through immediately)
    let mut camera = Camera::new(
        Vec3::new(0.0, 150.0, 0.0), // Spawn above max terrain height (140)
        Vec3::new(0.0, 1.0, 0.0),
        -90.0,
        0.0,
    );

    let mut world = World::new(seed);
    let render_distance = 6;
    let mut meshes: HashMap<(i32, i32), Mesh> = HashMap::new();

    // Initial Generation
    for x in -render_distance..=render_distance {
        for z in -render_distance..=render_distance {
            let chunk = world.get_chunk(x, z);
            let mesh = generate_chunk_mesh(chunk);
            println!("Chunk ({}, {}): {} vertices", x, z, mesh.index_count);
            meshes.insert((x, z), mesh);
        }
    }
    println!("Total Meshes: {}", meshes.len());

    let mut first_mouse = true;
    let mut last_x = SCR_WIDTH as f32 / 2.0;
    let mut last_y = SCR_HEIGHT as f32 / 2.0;
    let mut delta_time: f32;
    let mut last_frame: f32 = glfw.get_time() as f32;

    let mut is_flying = false; // Start in walking mode with collision
    let mut last_space_time = 0.0;
    let mut velocity = Vec3::ZERO;
    let gravity = -28.0; // Strong gravity
    let jump_force = 10.0;

    let mut held_block = BlockType::Stone;

    let mut is_paused = false;

    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;
        if delta_time > 0.1 {
            delta_time = 0.1;
        }

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    is_paused = !is_paused;
                    if is_paused {
                        window.set_cursor_mode(glfw::CursorMode::Normal);
                    } else {
                        window.set_cursor_mode(glfw::CursorMode::Disabled);
                        // Prevent camera jump
                        first_mouse = true;
                    }
                }
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) if !is_paused => {
                    if current_frame - last_space_time < 0.3 {
                        is_flying = !is_flying;
                        println!("Flying: {}", is_flying);
                        velocity = Vec3::ZERO;
                    } else if !is_flying {
                        // Jump if grounded (simple check: if velocity.y is 0 or low)
                        // For robust check, use raycast down or collision flag
                        velocity.y = jump_force;
                    }
                    last_space_time = current_frame;
                }
                glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) if !is_paused => {
                    held_block = BlockType::Dirt
                }
                glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) if !is_paused => {
                    held_block = BlockType::Stone
                }
                glfw::WindowEvent::Key(Key::Num3, _, Action::Press, _) if !is_paused => {
                    held_block = BlockType::Grass
                }
                glfw::WindowEvent::Key(Key::Num4, _, Action::Press, _) if !is_paused => {
                    held_block = BlockType::Water
                }
                glfw::WindowEvent::Key(Key::Num5, _, Action::Press, _) if !is_paused => {
                    held_block = BlockType::OakLog
                }
                glfw::WindowEvent::Key(Key::Num6, _, Action::Press, _) if !is_paused => {
                    held_block = BlockType::Leaves
                }

                glfw::WindowEvent::MouseButton(button, Action::Press, _) if !is_paused => {
                    if let Some(hit) = raycast(camera.position, camera.front, 5.0, &world) {
                        let (bx, by, bz) = hit.block_pos;
                        let mut changed_chunk = None; // (cx, cz)

                        if button == MouseButton::Button1 {
                            // Left: Break
                            world.set_block(bx, by, bz, Block::new(BlockType::Air));
                            changed_chunk = Some((bx, bz));
                        } else if button == MouseButton::Button2 {
                            // Right: Place
                            // Place at bx + face
                            let nx = bx + hit.face.0;
                            let ny = by + hit.face.1;
                            let nz = bz + hit.face.2;

                            // Don't place intersect player
                            let p_pos = camera.position;
                            // Simple box check
                            let p_min = p_pos - Vec3::new(0.3, 0.0, 0.3);
                            let p_max = p_pos + Vec3::new(0.3, 1.8, 0.3);
                            let b_min = Vec3::new(nx as f32, ny as f32, nz as f32);
                            let b_max = b_min + Vec3::ONE;

                            let intersect = p_min.x <= b_max.x
                                && p_max.x >= b_min.x
                                && p_min.y <= b_max.y
                                && p_max.y >= b_min.y
                                && p_min.z <= b_max.z
                                && p_max.z >= b_min.z;

                            if !intersect {
                                world.set_block(nx, ny, nz, Block::new(held_block));
                                changed_chunk = Some((nx, nz));
                            }
                        }

                        if let Some((x, z)) = changed_chunk {
                            let cx = (x as f32 / CHUNK_WIDTH as f32).floor() as i32;
                            let cz = (z as f32 / CHUNK_WIDTH as f32).floor() as i32;

                            if let Some(c) = world.get_chunk_ref(cx, cz) {
                                let m = generate_chunk_mesh(c);
                                meshes.insert((cx, cz), m);
                            }
                        }
                    }
                }

                glfw::WindowEvent::CursorPos(xpos, ypos) => {
                    let (xpos, ypos) = (xpos as f32, ypos as f32);
                    if first_mouse {
                        last_x = xpos;
                        last_y = ypos;
                        first_mouse = false;
                    }
                    if !is_paused {
                        camera.process_mouse_movement(xpos - last_x, last_y - ypos, true);
                    }
                    last_x = xpos;
                    last_y = ypos;
                }
                _ => {}
            }
        }

        // Movement Logic
        if !is_paused {
            let speed = if is_flying { 20.0 } else { 5.0 };
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
                // Flight movement with collision
                if window.get_key(Key::Space) == Action::Press {
                    move_dir += camera.world_up;
                }
                if window.get_key(Key::LeftShift) == Action::Press {
                    move_dir -= camera.world_up;
                }

                // Normalize to prevent faster diagonal
                if move_dir.length_squared() > 0.0 {
                    move_dir = move_dir.normalize();
                }

                // Apply movement with collision detection
                let displacement = move_dir * speed * delta_time;

                // X Axis
                let mut next_pos = camera.position;
                next_pos.x += displacement.x;
                if !check_collision(next_pos, &world) {
                    camera.position.x += displacement.x;
                }

                // Y Axis
                next_pos = camera.position;
                next_pos.y += displacement.y;
                if !check_collision(next_pos, &world) {
                    camera.position.y += displacement.y;
                }

                // Z Axis
                next_pos = camera.position;
                next_pos.z += displacement.z;
                if !check_collision(next_pos, &world) {
                    camera.position.z += displacement.z;
                }

                velocity = Vec3::ZERO;
            } else {
                // Walking & Gravity
                // Keep Y component separate
                move_dir.y = 0.0;
                if move_dir.length_squared() > 0.0 {
                    move_dir = move_dir.normalize();
                }

                // X/Z movement
                let target_vel = move_dir * speed;
                velocity.x = target_vel.x; // Instant acceleration for responsiveness
                velocity.z = target_vel.z;

                // Gravity
                velocity.y += gravity * delta_time;

                // Integrate
                let displacement = velocity * delta_time;

                // Collision Detection (Discrete)
                // X Axis
                let mut next_pos = camera.position;
                next_pos.x += displacement.x;
                if check_collision(next_pos, &world) {
                    // Halt X
                    velocity.x = 0.0;
                } else {
                    camera.position.x += displacement.x;
                }

                // Z Axis
                next_pos = camera.position;
                next_pos.z += displacement.z;
                if check_collision(next_pos, &world) {
                    velocity.z = 0.0;
                } else {
                    camera.position.z += displacement.z;
                }

                // Y Axis
                next_pos = camera.position;
                next_pos.y += displacement.y;
                if check_collision(next_pos, &world) {
                    velocity.y = 0.0; // Landed or hit head
                } else {
                    camera.position.y += displacement.y;
                }
            }
        }

        // Title Update
        let title = format!(
            "Voxel Engine | FPS: {:.0} | Pos: ({:.1}, {:.1}, {:.1}) | Block: {:?} | Chunks: {} | {}",
            1.0 / delta_time,
            camera.position.x,
            camera.position.y,
            camera.position.z,
            held_block,
            meshes.len(),
            if is_paused { "PAUSED" } else { "" }
        );
        window.set_title(&title);

        unsafe {
            gl::ClearColor(0.53, 0.81, 0.92, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        shader.use_program();
        texture.bind();

        let projection = Mat4::perspective_rh_gl(
            80.0_f32.to_radians(), // 80 FOV
            SCR_WIDTH as f32 / SCR_HEIGHT as f32,
            0.1,
            1000.0,
        );
        let view = camera.get_view_matrix();
        shader.set_mat4("projection", &projection);
        shader.set_mat4("view", &view);
        shader.set_vec3("lightDir", &Vec3::new(0.5, -1.0, 0.5));
        shader.set_vec3("viewPos", &camera.position);
        shader.set_int("ourTexture", 0); // Bind to texture unit 0

        shader.set_bool("isWater", false); // Default

        for ((cx, cz), mesh) in &meshes {
            let pos = Vec3::new(
                (*cx * CHUNK_WIDTH as i32) as f32,
                0.0,
                (*cz * CHUNK_WIDTH as i32) as f32,
            ); // Note: CHUNK_DEPTH assumed same as WIDTH usually, but be careful
            // Note: In Chunk we used WIDTH=16, DEPTH=16.
            // My mesh generation puts blocks at relative 0..15.
            // So I need to translate mesh by chunk world pos.

            let model = Mat4::from_translation(pos);
            shader.set_mat4("model", &model);
            mesh.draw();
        }

        // --- Crosshair Drawing (Immediate mode for simplicity in core profile requires shader, doing simple rect in framebuffer clears/logic is hard,
        // we will use a very simple glInvert logic on center pixels if possible or just assume user is happy with world for now)
        // User REALLY wants a crosshair.
        // Let's implement a quick crosshair using just a tiny quad in front of camera? No, that's complex.
        // We can use a separate shader for 2D.
        // Or... since we want it quick:
        // Let's manipulate the pixels directly? Too slow.
        // Better: Use `gl::Clear` on a small scissor box in the center?
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            let cx = (SCR_WIDTH / 2) as i32;
            let cy = (SCR_HEIGHT / 2) as i32;
            let size = 2; // Crosshair size
            gl::Scissor(cx - size, cy - size, 2 * size + 1, 2 * size + 1);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0); // White crosshair
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Disable(gl::SCISSOR_TEST);
        }

        window.swap_buffers();
    }
}
