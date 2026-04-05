mod camera;
mod entity;
mod fov;
mod game_state;
mod input;
mod iso;
mod pathfinding;
mod renderer;
mod tilemap;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use camera::Camera;
use game_state::GameState;
use input::GameInput;
use iso::screen_to_grid;
use tilemap::Tilemap;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 900;
const WINDOW_TITLE: &str = "Isometric Game";

const TICKS_PER_SECOND: u32 = 60;
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND as u64);

fn main() {
    // --- Load map and create game state ---
    let tilemap = Tilemap::from_file("assets/map.json").expect("Failed to load map");
    let mut state = GameState::new(tilemap);

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

    // --- Client-only state (not part of GameState) ---
    let mut camera = Camera::new();
    let mut previous_time = Instant::now();
    let mut lag = Duration::ZERO;
    let mut running = true;
    let mut fps_timer = Instant::now();
    let mut frame_count: u32 = 0;

    // --- Game Loop ---
    while running {
        let current_time = Instant::now();
        let elapsed = current_time - previous_time;
        previous_time = current_time;
        lag += elapsed;

        // 1. PROCESS INPUT — translate SDL2 events to GameInput
        let player_id = state.local_player_id;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown { scancode: Some(Scancode::Escape), .. } => running = false,
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    let (ww, wh) = canvas.output_size().unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
                    let world_x = x - (ww as i32 / 2) + camera.x;
                    let world_y = y - (wh as i32 / 4) + camera.y;
                    let (grid_x, grid_y) = screen_to_grid(world_x, world_y);

                    if grid_x >= 0 && grid_x < state.tilemap.cols
                        && grid_y >= 0 && grid_y < state.tilemap.rows
                    {
                        state.apply_input(GameInput::MoveTo {
                            entity_id: player_id,
                            target_x: grid_x,
                            target_y: grid_y,
                        });
                    }
                }
                _ => {}
            }
        }

        // WASD input (held keys)
        let keyboard = event_pump.keyboard_state();
        let (dx, dy) = if keyboard.is_scancode_pressed(Scancode::W) || keyboard.is_scancode_pressed(Scancode::Up) {
            (0, -1)
        } else if keyboard.is_scancode_pressed(Scancode::S) || keyboard.is_scancode_pressed(Scancode::Down) {
            (0, 1)
        } else if keyboard.is_scancode_pressed(Scancode::A) || keyboard.is_scancode_pressed(Scancode::Left) {
            (-1, 0)
        } else if keyboard.is_scancode_pressed(Scancode::D) || keyboard.is_scancode_pressed(Scancode::Right) {
            (1, 0)
        } else {
            (0, 0)
        };

        // 2. UPDATE — fixed timestep
        while lag >= TICK_DURATION {
            if dx != 0 || dy != 0 {
                state.apply_input(GameInput::MoveDirection {
                    entity_id: player_id,
                    dx,
                    dy,
                });
            }

            state.tick();

            // Camera follows local player's visual position
            if let Some(player) = state.local_player() {
                camera.x = player.visual_x as i32;
                camera.y = player.visual_y as i32;
            }

            lag -= TICK_DURATION;
        }

        // 3. RENDER
        canvas.set_draw_color(Color::RGB(20, 20, 40));
        canvas.clear();

        renderer::draw_world(&mut canvas, &state, &camera);

        canvas.present();

        // FPS counter
        frame_count += 1;
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            canvas.window_mut().set_title(&format!("{WINDOW_TITLE} — {frame_count} FPS")).ok();
            frame_count = 0;
            fps_timer = Instant::now();
        }
    }

    println!("Game closed");
}
