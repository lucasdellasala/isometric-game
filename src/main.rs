mod camera;
mod iso;
mod player;
mod renderer;
mod tilemap;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use camera::Camera;
use player::Player;
use tilemap::Tilemap;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const WINDOW_TITLE: &str = "Isometric Game";

// Fixed timestep: 60 logic updates per second
const TICKS_PER_SECOND: u32 = 60;
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND as u64);

// Player movement: one step every N ticks (prevents moving too fast)
const MOVE_COOLDOWN: u32 = 6;

fn main() {
    // --- Load map from file ---
    let tilemap = Tilemap::from_file("assets/map.json").expect("Failed to load map");

    // --- Initialize SDL2 ---
    let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
    let video_subsystem = sdl_context.video().expect("Failed to initialize video");

    let window = video_subsystem
        .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
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
    let mut move_timer: u32 = 0;
    let mut previous_time = Instant::now();
    let mut lag = Duration::ZERO;
    let mut running = true;

    // FPS counter
    let mut fps_timer = Instant::now();
    let mut frame_count: u32 = 0;

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
                _ => {}
            }
        }

        // 2. UPDATE LOGIC (fixed timestep)
        let keyboard = event_pump.keyboard_state();

        while lag >= TICK_DURATION {
            // Player movement with cooldown
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

            // Camera follows player
            let (target_x, target_y) = iso::grid_to_screen(player.grid_x, player.grid_y);
            camera.x = target_x;
            camera.y = target_y;

            lag -= TICK_DURATION;
        }

        // 3. RENDER
        canvas.set_draw_color(Color::RGB(20, 20, 40));
        canvas.clear();

        renderer::draw_world(&mut canvas, &tilemap, &player, &camera);

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
