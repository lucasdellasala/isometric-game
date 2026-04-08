mod core;
mod render;
mod ui;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use render::assets::AssetManager;
use render::camera::{Camera, CAMERA_ZOOM};
use render::iso::screen_to_grid;
use render::text::TextRenderer;

use core::game_state::GameState;
use core::input::GameInput;
use core::tilemap::Tilemap;

use ui::config_menu::ConfigMenu;
use ui::sprite_debug::SpriteDebug;

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

    // --- Asset Manager ---
    let texture_creator = canvas.texture_creator();
    let mut assets = AssetManager::new(&texture_creator);
    assets.generate_placeholders().expect("Failed to generate placeholder textures");

    // --- Text Renderer ---
    let mut text_renderer = TextRenderer::new(&texture_creator, "assets/fonts/default.ttf")
        .expect("Failed to load font");

    // --- Client-only state ---
    let mut camera = Camera::new();
    let mut previous_time = Instant::now();
    let mut lag = Duration::ZERO;
    let mut running = true;
    let mut fps_timer = Instant::now();
    let mut frame_count: u32 = 0;
    let mut config_menu = ConfigMenu::new();
    let mut sprite_debug = SpriteDebug::new(
        render::renderer::ENTITY_OFFSET_X,
        render::renderer::ENTITY_OFFSET_Y,
    );

    // --- Game Loop ---
    while running {
        let current_time = Instant::now();
        let elapsed = current_time - previous_time;
        previous_time = current_time;
        lag += elapsed;

        // 1. PROCESS INPUT
        let player_id = state.local_player_id;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown { scancode: Some(Scancode::Escape), .. } => {
                    if config_menu.visible {
                        config_menu.visible = false;
                    } else {
                        running = false;
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::F1), .. } => {
                    config_menu.toggle();
                }
                Event::KeyDown { scancode: Some(Scancode::F2), .. } => {
                    sprite_debug.toggle();
                }
                // When sprite debug is active, arrows adjust offsets, Tab toggles mode
                Event::KeyDown { scancode: Some(sc), .. } if sprite_debug.active => {
                    let facing = state.local_player()
                        .map(|p| p.facing)
                        .unwrap_or(0);
                    match sc {
                        Scancode::Up => sprite_debug.adjust(facing, 0, -1),
                        Scancode::Down => sprite_debug.adjust(facing, 0, 1),
                        Scancode::Left => sprite_debug.adjust(facing, -1, 0),
                        Scancode::Right => sprite_debug.adjust(facing, 1, 0),
                        Scancode::Tab => sprite_debug.toggle_mode(),
                        _ => {}
                    }
                }
                // When config menu is open, arrow keys navigate it
                Event::KeyDown { scancode: Some(sc), .. } if config_menu.visible => {
                    match sc {
                        Scancode::Up => config_menu.move_up(),
                        Scancode::Down => config_menu.move_down(),
                        Scancode::Left => config_menu.adjust_left(),
                        Scancode::Right => config_menu.adjust_right(),
                        _ => {}
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::E), .. } => {
                    if state.active_dialogue.is_some() {
                        state.apply_input(GameInput::DismissDialogue);
                    } else {
                        state.apply_input(GameInput::Interact {
                            entity_id: player_id,
                        });
                    }
                }
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    if state.active_dialogue.is_none() && !config_menu.visible {
                        let (ww, wh) = canvas.output_size().unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
                        // Undo the zoom applied in renderer::to_screen
                        let world_x = ((x - ww as i32 / 2) as f64 / CAMERA_ZOOM) as i32 + camera.x;
                        let world_y = ((y - wh as i32 / 4) as f64 / CAMERA_ZOOM) as i32 + camera.y;
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
                }
                _ => {}
            }
        }

        // WASD input (held keys) — blocked during dialogue or when menu is open
        let keyboard = event_pump.keyboard_state();
        let (dx, dy) = if state.active_dialogue.is_some() || config_menu.visible {
            (0, 0)
        } else if keyboard.is_scancode_pressed(Scancode::W) {
            (0, -1)
        } else if keyboard.is_scancode_pressed(Scancode::S) {
            (0, 1)
        } else if keyboard.is_scancode_pressed(Scancode::A) {
            (-1, 0)
        } else if keyboard.is_scancode_pressed(Scancode::D) {
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

            if let Some(player) = state.local_player() {
                camera.x = player.visual_x as i32;
                camera.y = player.visual_y as i32;
            }

            lag -= TICK_DURATION;
        }

        // 3. RENDER
        canvas.set_draw_color(Color::RGB(20, 20, 40));
        canvas.clear();

        // Compute hover tile from current mouse position
        let mouse_state = event_pump.mouse_state();
        let (ww, wh) = canvas.output_size().unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
        let hover_world_x = ((mouse_state.x() - ww as i32 / 2) as f64 / CAMERA_ZOOM) as i32 + camera.x;
        let hover_world_y = ((mouse_state.y() - wh as i32 / 4) as f64 / CAMERA_ZOOM) as i32 + camera.y;
        let (hgx, hgy) = screen_to_grid(hover_world_x, hover_world_y);
        let hover_tile = if hgx >= 0 && hgx < state.tilemap.cols && hgy >= 0 && hgy < state.tilemap.rows {
            Some((hgx, hgy))
        } else {
            None
        };

        let dither_params = config_menu.dither_params();
        let moebius_params = config_menu.moebius_params();

        render::renderer::render_frame(
            &mut canvas,
            &state,
            &camera,
            &mut assets,
            &mut text_renderer,
            config_menu.mode,
            config_menu.scope,
            Some(&dither_params),
            Some(&moebius_params),
            &sprite_debug,
            hover_tile,
        );

        // Config menu overlay (always on top, never post-processed)
        config_menu.draw(&mut canvas, &mut text_renderer);

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
