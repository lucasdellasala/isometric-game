mod camera;
mod iso;
mod renderer;
mod tilemap;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use camera::Camera;
use tilemap::Tilemap;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const WINDOW_TITLE: &str = "Isometric Game";

// Fixed timestep: 60 logic updates per second
const TICKS_PER_SECOND: u32 = 60;
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND as u64);

// Map size in tiles
const MAP_COLS: i32 = 16;
const MAP_ROWS: i32 = 16;

fn main() {
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
    let tilemap = Tilemap::new_test(MAP_COLS, MAP_ROWS);
    let mut previous_time = Instant::now();
    let mut lag = Duration::ZERO;
    let mut running = true;

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
            if keyboard.is_scancode_pressed(Scancode::Up) || keyboard.is_scancode_pressed(Scancode::W) {
                camera.move_by(0, -1);
            }
            if keyboard.is_scancode_pressed(Scancode::Down) || keyboard.is_scancode_pressed(Scancode::S) {
                camera.move_by(0, 1);
            }
            if keyboard.is_scancode_pressed(Scancode::Left) || keyboard.is_scancode_pressed(Scancode::A) {
                camera.move_by(-1, 0);
            }
            if keyboard.is_scancode_pressed(Scancode::Right) || keyboard.is_scancode_pressed(Scancode::D) {
                camera.move_by(1, 0);
            }

            lag -= TICK_DURATION;
        }

        // 3. RENDER
        canvas.set_draw_color(Color::RGB(20, 20, 40));
        canvas.clear();

        renderer::draw_tilemap(&mut canvas, &tilemap, &camera);

        canvas.present();
    }

    println!("Game closed");
}
