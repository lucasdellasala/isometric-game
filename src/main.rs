mod camera;
mod iso;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::time::{Duration, Instant};

use camera::Camera;
use iso::{grid_to_screen, TILE_HEIGHT, TILE_WIDTH};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const WINDOW_TITLE: &str = "Isometric Game";

// Fixed timestep: 60 logic updates per second
const TICKS_PER_SECOND: u32 = 60;
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND as u64);

// Map size in tiles
const MAP_COLS: i32 = 16;
const MAP_ROWS: i32 = 16;

/// Draw a single isometric diamond tile (outline) at the given grid position.
fn draw_tile(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, grid_x: i32, grid_y: i32, cam: &Camera, color: Color) {
    let (sx, sy) = grid_to_screen(grid_x, grid_y);

    // Apply camera offset and center on screen
    let cx = sx - cam.x + (WINDOW_WIDTH as i32 / 2);
    let cy = sy - cam.y + (WINDOW_HEIGHT as i32 / 4);

    // Diamond shape: 4 points (top, right, bottom, left)
    let half_w = TILE_WIDTH / 2;
    let half_h = TILE_HEIGHT / 2;

    let top = Point::new(cx, cy);
    let right = Point::new(cx + half_w, cy + half_h);
    let bottom = Point::new(cx, cy + TILE_HEIGHT);
    let left = Point::new(cx - half_w, cy + half_h);

    canvas.set_draw_color(color);
    // Draw 4 lines forming the diamond
    let _ = canvas.draw_line(top, right);
    let _ = canvas.draw_line(right, bottom);
    let _ = canvas.draw_line(bottom, left);
    let _ = canvas.draw_line(left, top);
}

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
        // Check which keys are currently held down
        let keyboard = event_pump.keyboard_state();

        while lag >= TICK_DURATION {
            // Camera movement based on held keys
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

        // Draw isometric grid
        for row in 0..MAP_ROWS {
            for col in 0..MAP_COLS {
                let color = if (col + row) % 2 == 0 {
                    Color::RGB(60, 120, 60)  // Dark green
                } else {
                    Color::RGB(80, 150, 80)  // Light green
                };
                draw_tile(&mut canvas, col, row, &camera, color);
            }
        }

        canvas.present();
    }

    println!("Game closed");
}
