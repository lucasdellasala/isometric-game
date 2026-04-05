use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const WINDOW_TITLE: &str = "Isometric Game";

// Fixed timestep: 60 logic updates per second
const TICKS_PER_SECOND: u32 = 60;
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND as u64);

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

    // --- Game loop variables ---
    let mut previous_time = Instant::now();
    let mut lag = Duration::ZERO;
    let mut tick_count: u64 = 0;
    let mut running = true;

    // --- Game Loop ---
    while running {
        let current_time = Instant::now();
        let elapsed = current_time - previous_time;
        previous_time = current_time;
        lag += elapsed;

        // 1. PROCESS INPUT
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    running = false;
                }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    running = false;
                }
                _ => {}
            }
        }

        // 2. UPDATE LOGIC (fixed timestep)
        while lag >= TICK_DURATION {
            // Game logic goes here: move characters, physics, AI, etc.
            tick_count += 1;
            lag -= TICK_DURATION;
        }

        // 3. RENDER
        canvas.set_draw_color(Color::RGB(20, 20, 40));
        canvas.clear();
        // Rendering goes here: draw tiles, sprites, UI, etc.
        canvas.present();
    }

    println!("Game closed after {tick_count} ticks");
}
