mod camera;
mod fov;
mod iso;
mod pathfinding;
mod player;
mod renderer;
mod tilemap;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use camera::Camera;
use fov::FovMap;
use iso::screen_to_grid;
use player::Player;
use tilemap::Tilemap;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 900;
const WINDOW_TITLE: &str = "Isometric Game";

// Fixed timestep: 60 logic updates per second
const TICKS_PER_SECOND: u32 = 60;
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND as u64);

// Player movement: one step every N ticks (prevents moving too fast when using WASD)
const MOVE_COOLDOWN: u32 = 6;

// FOV radius in tiles
const FOV_RADIUS: i32 = 10;

fn main() {
    // --- Load map from file ---
    let tilemap = Tilemap::from_file("assets/map.json").expect("Failed to load map");

    // --- Initialize SDL2 ---
    let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
    let video_subsystem = sdl_context.video().expect("Failed to initialize video");

    let window = video_subsystem
        .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .maximized()
        .build()
        .expect("Failed to create window");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("Failed to create canvas");

    let mut event_pump = sdl_context.event_pump().expect("Failed to get event pump");

    // --- Game state ---
    let mut camera = Camera::new();
    let mut player = Player::new(0, 0);
    let mut fov_map = FovMap::new(tilemap.cols, tilemap.rows);
    let mut move_timer: u32 = 0;
    let mut click_target: Option<(i32, i32)> = None;
    let mut previous_time = Instant::now();
    let mut lag = Duration::ZERO;
    let mut running = true;

    // FPS counter
    let mut fps_timer = Instant::now();
    let mut frame_count: u32 = 0;

    // Initial FOV computation
    fov_map.compute(player.grid_x, player.grid_y, FOV_RADIUS, &tilemap);

    // --- Game Loop ---
    while running {
        let current_time = Instant::now();
        let elapsed = current_time - previous_time;
        previous_time = current_time;
        lag += elapsed;

        // 1. PROCESS INPUT (events)
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    running = false;
                }
                Event::KeyDown { scancode: Some(Scancode::Escape), .. } => {
                    running = false;
                }
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    let (ww, wh) = canvas.output_size().unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
                    let world_x = x - (ww as i32 / 2) + camera.x;
                    let world_y = y - (wh as i32 / 4) + camera.y;
                    let (grid_x, grid_y) = screen_to_grid(world_x, world_y);

                    if grid_x >= 0 && grid_x < tilemap.cols && grid_y >= 0 && grid_y < tilemap.rows {
                        player.move_to(grid_x, grid_y, &tilemap);
                        click_target = Some((grid_x, grid_y));
                    }
                }
                _ => {}
            }
        }

        // 2. UPDATE LOGIC (fixed timestep)
        let keyboard = event_pump.keyboard_state();

        while lag >= TICK_DURATION {
            // Clear marker when player arrives
            if !player.is_walking() && click_target.is_some() {
                click_target = None;
            }

            // WASD overrides pathfinding
            if !player.is_walking() {
                if move_timer > 0 {
                    move_timer -= 1;
                } else {
                    let mut moved = false;

                    if keyboard.is_scancode_pressed(Scancode::W) || keyboard.is_scancode_pressed(Scancode::Up) {
                        player.try_move(0, -1, &tilemap);
                        moved = true;
                    } else if keyboard.is_scancode_pressed(Scancode::S) || keyboard.is_scancode_pressed(Scancode::Down) {
                        player.try_move(0, 1, &tilemap);
                        moved = true;
                    } else if keyboard.is_scancode_pressed(Scancode::A) || keyboard.is_scancode_pressed(Scancode::Left) {
                        player.try_move(-1, 0, &tilemap);
                        moved = true;
                    } else if keyboard.is_scancode_pressed(Scancode::D) || keyboard.is_scancode_pressed(Scancode::Right) {
                        player.try_move(1, 0, &tilemap);
                        moved = true;
                    }

                    if moved {
                        move_timer = MOVE_COOLDOWN;
                    }
                }
            }

            // Advance pathfinding + smooth visual interpolation
            player.update();

            // Camera follows player's visual position (smooth)
            camera.x = player.visual_x as i32;
            camera.y = player.visual_y as i32;

            // Recompute FOV every tick (cheap for small radius, avoids visual jumps)
            fov_map.compute(player.grid_x, player.grid_y, FOV_RADIUS, &tilemap);

            lag -= TICK_DURATION;
        }

        // 3. RENDER
        canvas.set_draw_color(Color::RGB(20, 20, 40));
        canvas.clear();

        renderer::draw_world(&mut canvas, &tilemap, &player, &camera, click_target, &fov_map);

        canvas.present();

        // Update FPS counter every second
        frame_count += 1;
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            canvas.window_mut().set_title(&format!("{WINDOW_TITLE} — {frame_count} FPS")).ok();
            frame_count = 0;
            fps_timer = Instant::now();
        }
    }

    println!("Game closed");
}
