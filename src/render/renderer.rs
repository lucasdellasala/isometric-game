use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::render::assets::AssetManager;
use crate::render::camera::{Camera, CAMERA_ZOOM};
use crate::render::post_process::{self, ApplyScope, DitherParams, MoebiusParams, PostProcessMode};
use crate::core::entity::{Entity, EntityKind};
use crate::ui::sprite_debug::SpriteDebug;
use crate::core::game_state::GameState;
use crate::render::iso::{grid_to_screen, TILE_HEIGHT, TILE_WIDTH};
use crate::render::text::TextRenderer;
#[allow(unused_imports)]
use crate::core::tilemap::TileKind;

const CULL_MARGIN: i32 = TILE_WIDTH;

/// Manual offset to adjust entity sprite positioning on the tile.
/// Tweak these if the sprite has transparent padding that shifts it off-center.
/// Positive X = move sprite right, Positive Y = move sprite down.
pub const ENTITY_OFFSET_X: i32 = 2;
pub const ENTITY_OFFSET_Y: i32 = 30;

/// Scale factor for entity sprites relative to CAMERA_ZOOM.
/// 1.0 = full size, 0.66 = two thirds.
const ENTITY_SCALE: f64 = 0.66;

/// Convert grid position to screen position with camera offset and zoom applied.
fn to_screen(grid_x: i32, grid_y: i32, cam: &Camera, screen_w: i32, screen_h: i32) -> (i32, i32) {
    let (sx, sy) = grid_to_screen(grid_x, grid_y);
    let x = ((sx - cam.x) as f64 * CAMERA_ZOOM) as i32 + screen_w / 2;
    let y = ((sy - cam.y) as f64 * CAMERA_ZOOM) as i32 + screen_h / 4;
    (x, y)
}

/// Check if a screen position is visible (within the window + margin).
fn is_on_screen(cx: i32, cy: i32, screen_w: i32, screen_h: i32) -> bool {
    cx > -CULL_MARGIN - TILE_WIDTH
        && cx < screen_w + CULL_MARGIN + TILE_WIDTH
        && cy > -CULL_MARGIN - TILE_HEIGHT * 2
        && cy < screen_h + CULL_MARGIN + TILE_HEIGHT
}

/// Draw a hover highlight (diamond outline) on the tile under the mouse cursor.
fn draw_hover(canvas: &mut Canvas<Window>, grid_x: i32, grid_y: i32, cam: &Camera, sw: i32, sh: i32) {
    let (cx, cy) = to_screen(grid_x, grid_y, cam, sw, sh);
    let hw = (TILE_WIDTH as f64 * CAMERA_ZOOM / 2.0) as i32;
    let hh = (TILE_HEIGHT as f64 * CAMERA_ZOOM / 2.0) as i32;
    let mid_y = cy + hh;

    canvas.set_draw_color(Color::RGBA(255, 255, 255, 120));
    let _ = canvas.draw_line(Point::new(cx, cy), Point::new(cx + hw, mid_y));
    let _ = canvas.draw_line(Point::new(cx + hw, mid_y), Point::new(cx, cy + hh * 2));
    let _ = canvas.draw_line(Point::new(cx, cy + hh * 2), Point::new(cx - hw, mid_y));
    let _ = canvas.draw_line(Point::new(cx - hw, mid_y), Point::new(cx, cy));
}

/// Draw a target marker (small yellow diamond) on a tile.
fn draw_marker(canvas: &mut Canvas<Window>, grid_x: i32, grid_y: i32, cam: &Camera, sw: i32, sh: i32) {
    let (cx, cy) = to_screen(grid_x, grid_y, cam, sw, sh);
    let size = (6.0 * CAMERA_ZOOM) as i32;
    let center_y = cy + (TILE_HEIGHT as f64 * CAMERA_ZOOM) as i32 / 2;

    canvas.set_draw_color(Color::RGB(255, 255, 0));
    for y in 0..size {
        let w = if y < size / 2 { y } else { size - y };
        let _ = canvas.draw_line(
            Point::new(cx - w, center_y - size / 2 + y),
            Point::new(cx + w, center_y - size / 2 + y),
        );
    }
}

/// Get the texture key for a tile kind.
fn tile_texture_key(kind: TileKind) -> &'static str {
    match kind {
        TileKind::Grass => "tile_grass",
        TileKind::Dirt => "tile_dirt",
        TileKind::Stone => "tile_stone",
        TileKind::Water => "tile_water",
    }
}

/// Get the texture key for an entity, considering facing and walk animation.
fn entity_texture_key(entity: &Entity) -> String {
    match entity.kind {
        EntityKind::Player => {
            if let Some(frame) = entity.walk_frame() {
                format!("entity_player_walk_{:03}_{}", entity.facing, frame)
            } else {
                format!("entity_player_{:03}", entity.facing)
            }
        }
        EntityKind::Npc => String::from("entity_npc"),
        EntityKind::Enemy => String::from("entity_enemy"),
    }
}

/// Draw a tile texture at a screen position, darkened by FOV brightness, scaled by zoom.
fn draw_tile(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    key: &str,
    cx: i32,
    cy: i32,
    brightness: f64,
    zoom: f64,
) {
    if let Some(texture) = assets.get_mut(key) {
        let query = texture.query();
        let w = (query.width as f64 * zoom) as u32;
        let h = (query.height as f64 * zoom) as u32;
        let dst = Rect::new(
            cx - w as i32 / 2,
            cy,
            w,
            h,
        );

        // Darken texture using color mod (only affects non-transparent pixels)
        let b = (brightness * 255.0) as u8;
        texture.set_color_mod(b, b, b);
        let _ = canvas.copy(texture, None, dst);
    }
}

/// Draw an entity sprite at its visual position.
fn draw_entity(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    entity: &Entity,
    fov_map: &crate::core::fov::FovMap,
    cam: &Camera,
    sw: i32,
    sh: i32,
    sprite_debug: &SpriteDebug,
    text: &mut TextRenderer,
) {
    // Don't draw entities in unexplored or dark areas
    let brightness = fov_map.get_brightness(entity.grid_x, entity.grid_y);
    if brightness < 0.5 {
        return;
    }

    let cx = ((entity.visual_x as i32 - cam.x) as f64 * CAMERA_ZOOM) as i32 + sw / 2;
    let cy = ((entity.visual_y as i32 - cam.y) as f64 * CAMERA_ZOOM) as i32 + sh / 4;

    let key = entity_texture_key(entity);

    if let Some(texture) = assets.get_mut(&key) {
        let query = texture.query();
        let entity_zoom = CAMERA_ZOOM * ENTITY_SCALE;
        let w = (query.width as f64 * entity_zoom) as u32;
        let h = (query.height as f64 * entity_zoom) as u32;
        let th = (TILE_HEIGHT as f64 * CAMERA_ZOOM) as i32;

        // Player always uses sprite_debug offsets (persisted between debug sessions).
        // Other entities use the constants.
        let (off_x, off_y) = if entity.kind == EntityKind::Player {
            let (dx, dy) = sprite_debug.get_offset(entity.facing);
            ((dx as f64 * CAMERA_ZOOM) as i32, (dy as f64 * CAMERA_ZOOM) as i32)
        } else {
            let ox = (ENTITY_OFFSET_X as f64 * CAMERA_ZOOM) as i32;
            let oy = (ENTITY_OFFSET_Y as f64 * CAMERA_ZOOM) as i32;
            (ox, oy)
        };

        let dst = Rect::new(
            cx - w as i32 / 2 + off_x,
            cy + th / 2 - h as i32 + off_y,
            w,
            h,
        );
        let _ = canvas.copy(texture, None, dst);

        // Show sprite key label above player when debug is active
        if sprite_debug.active && entity.kind == EntityKind::Player {
            let (raw_x, raw_y) = sprite_debug.get_offset(entity.facing);
            let label = format!("{} | offset({},{})", key, raw_x, raw_y);
            if let Some(tex) = text.render(&label, 14, Color::RGB(255, 255, 0)) {
                let q = tex.query();
                let label_dst = Rect::new(
                    cx - q.width as i32 / 2,
                    dst.y() - q.height as i32 - 4,
                    q.width,
                    q.height,
                );
                let _ = canvas.copy(tex, None, label_dst);
            }

            // Show mode indicator
            let mode_label = if sprite_debug.per_direction_mode {
                format!("[TAB] Mode: per-direction ({:03})", entity.facing)
            } else {
                "[TAB] Mode: base offset".to_string()
            };
            if let Some(tex) = text.render(&mode_label, 12, Color::RGB(200, 200, 200)) {
                let q = tex.query();
                let mode_dst = Rect::new(
                    cx - q.width as i32 / 2,
                    dst.y() - q.height as i32 - 22,
                    q.width,
                    q.height,
                );
                let _ = canvas.copy(tex, None, mode_dst);
            }
        }
    }
}

/// Draw an isometric wall cube at a grid position.
/// height_tiles = how many tile heights tall the wall is.
/// Sides are solid gray, top is the stone tile texture.
fn draw_wall_cube(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    cx: i32,
    cy: i32,
    height_tiles: i32,
    brightness: f64,
) {
    let hw = (TILE_WIDTH as f64 * CAMERA_ZOOM / 2.0) as i32;
    let hh = (TILE_HEIGHT as f64 * CAMERA_ZOOM / 2.0) as i32;
    let cube_h = (TILE_HEIGHT as f64 * CAMERA_ZOOM * height_tiles as f64) as i32;

    let top_y = cy - cube_h;
    let top_center = top_y + hh;

    // Left face: solid gray, slightly lighter
    let left_b = (brightness * 0.7).min(1.0);
    let left_color = Color::RGB((130.0 * left_b) as u8, (130.0 * left_b) as u8, (135.0 * left_b) as u8);
    canvas.set_draw_color(left_color);
    for h in 0..cube_h {
        let _ = canvas.draw_line(
            Point::new(cx - hw, top_center + h),
            Point::new(cx, top_y + hh * 2 + h),
        );
    }

    // Right face: solid gray, darker
    let right_b = (brightness * 0.5).min(1.0);
    let right_color = Color::RGB((110.0 * right_b) as u8, (110.0 * right_b) as u8, (115.0 * right_b) as u8);
    canvas.set_draw_color(right_color);
    for h in 0..cube_h {
        let _ = canvas.draw_line(
            Point::new(cx, top_y + hh * 2 + h),
            Point::new(cx + hw, top_center + h),
        );
    }

    // Top face: stone tile texture
    draw_tile(canvas, assets, "tile_stone", cx, top_y, brightness, CAMERA_ZOOM);
}

/// Compute the isometric depth row for an entity.
/// Uses the max of grid depth and visual depth (ceiling) to avoid flickering:
/// - Moving down-right: grid depth is higher → entity drawn after destination tiles
/// - Moving up-left: visual depth (ceil) stays high → entity stays at old row until arrival
/// Ceiling division ensures the entity isn't drawn before tiles it visually overlaps.
fn entity_depth_row(entity: &Entity) -> i32 {
    let grid_depth = entity.grid_x + entity.grid_y;
    let half_tile = TILE_HEIGHT / 2;
    // Ceiling division: (visual_y + half_tile - 1) / half_tile
    let visual_depth = (entity.visual_y as i32 + half_tile - 1) / half_tile;
    grid_depth.max(visual_depth)
}

/// Draw only the tilemap (ground tiles). No entities.
/// Call this first, then optionally apply dithering, then call draw_entities.
pub fn draw_tiles(canvas: &mut Canvas<Window>, state: &GameState, cam: &Camera, assets: &mut AssetManager) {
    let (sw, sh) = canvas.output_size().unwrap_or((1280, 900));
    let sw = sw as i32;
    let sh = sh as i32;

    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    let max_depth = state.tilemap.cols + state.tilemap.rows - 2;

    for depth in 0..=max_depth {
        let col_min = (depth - state.tilemap.rows + 1).max(0);
        let col_max = depth.min(state.tilemap.cols - 1);

        for col in col_min..=col_max {
            let row = depth - col;
            let (cx, cy) = to_screen(col, row, cam, sw, sh);

            if !is_on_screen(cx, cy, sw, sh) {
                continue;
            }

            let dim = state.fov_map.get_brightness(col, row);
            if dim < 0.01 {
                continue;
            }

            let tile = state.tilemap.get(col, row);
            let key = tile_texture_key(tile);
            draw_tile(canvas, assets, key, cx, cy, dim, CAMERA_ZOOM);

            // Test wall cube at grid position (1, 0), 2 tiles tall
            if col == 1 && row == 0 {
                draw_wall_cube(canvas, assets, cx, cy, 2, dim);
            }
        }
    }
}

/// Draw entities and UI overlays. Call after draw_tiles (and optional dithering).
/// Entities are depth-sorted and drawn with full color.
pub fn draw_entities_and_ui(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    cam: &Camera,
    assets: &mut AssetManager,
    text: &mut TextRenderer,
    sprite_debug: &SpriteDebug,
    hover_tile: Option<(i32, i32)>,
) {
    let (sw, sh) = canvas.output_size().unwrap_or((1280, 900));
    let sw = sw as i32;
    let sh = sh as i32;

    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    // Pre-compute depth row for each entity
    let mut entity_draw_order: Vec<(i32, usize)> = state.entities
        .iter()
        .enumerate()
        .map(|(idx, e)| (entity_depth_row(e), idx))
        .collect();
    entity_draw_order.sort_by_key(|(depth, _)| *depth);

    for &(_, entity_idx) in &entity_draw_order {
        draw_entity(canvas, assets, &state.entities[entity_idx], &state.fov_map, cam, sw, sh, sprite_debug, text);
    }

    // Draw hover highlight on tile under mouse
    if let Some((hx, hy)) = hover_tile {
        draw_hover(canvas, hx, hy, cam, sw, sh);
    }

    // Draw click target marker
    if let Some((tx, ty)) = state.click_target {
        if state.fov_map.get_brightness(tx, ty) > 0.5 {
            draw_marker(canvas, tx, ty, cam, sw, sh);
        }
    }

    // Draw dialogue box if active
    if let Some(dialogue) = &state.active_dialogue {
        draw_dialogue_box(canvas, text, &dialogue.target_name, &dialogue.text, sw, sh);
    }
}

/// Draw a dialogue box at the bottom of the screen.
/// Shows the speaker name and their text in a semi-transparent box.
fn draw_dialogue_box(
    canvas: &mut Canvas<Window>,
    text: &mut TextRenderer,
    speaker: &str,
    dialogue_text: &str,
    sw: i32,
    sh: i32,
) {
    let box_height = 120;
    let margin = 20;
    let padding = 16;

    // Semi-transparent dark background
    canvas.set_draw_color(Color::RGBA(10, 10, 30, 200));
    let box_rect = Rect::new(margin, sh - box_height - margin, (sw - margin * 2) as u32, box_height as u32);
    let _ = canvas.fill_rect(box_rect);

    // Border
    canvas.set_draw_color(Color::RGB(180, 160, 100));
    let _ = canvas.draw_rect(box_rect);

    // Speaker name (yellow, larger font)
    let name_x = margin + padding;
    let name_y = sh - box_height - margin + padding;
    if let Some(name_tex) = text.render(speaker, 22, Color::RGB(255, 220, 100)) {
        let q = name_tex.query();
        let dst = Rect::new(name_x, name_y, q.width, q.height);
        let _ = canvas.copy(name_tex, None, dst);
    }

    // Dialogue text (white, smaller font)
    let text_x = margin + padding;
    let text_y = name_y + 30;
    if let Some(text_tex) = text.render(dialogue_text, 18, Color::RGB(230, 230, 230)) {
        let q = text_tex.query();
        let dst = Rect::new(text_x, text_y, q.width, q.height);
        let _ = canvas.copy(text_tex, None, dst);
    }

    // Hint text
    let hint = "[E] Cerrar";
    let hint_x = margin + padding;
    let hint_y = sh - margin - padding - 16;
    if let Some(hint_tex) = text.render(hint, 14, Color::RGB(150, 150, 150)) {
        let q = hint_tex.query();
        let dst = Rect::new(hint_x, hint_y, q.width, q.height);
        let _ = canvas.copy(hint_tex, None, dst);
    }
}

/// Orchestrate the full render pass: tiles, optional post-process, entities+UI.
/// Centralizes the render pipeline so main.rs stays thin.
pub fn render_frame(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    cam: &Camera,
    assets: &mut AssetManager,
    text: &mut TextRenderer,
    mode: PostProcessMode,
    scope: ApplyScope,
    dither_params: Option<&DitherParams>,
    moebius_params: Option<&MoebiusParams>,
    sprite_debug: &SpriteDebug,
    hover_tile: Option<(i32, i32)>,
) {
    match mode {
        PostProcessMode::Off => {
            draw_tiles(canvas, state, cam, assets);
            draw_entities_and_ui(canvas, state, cam, assets, text, sprite_debug, hover_tile);
        }
        PostProcessMode::Dithering => {
            if let Some(params) = dither_params {
                match scope {
                    ApplyScope::TilesOnly => {
                        draw_tiles(canvas, state, cam, assets);
                        post_process::apply_dither(canvas, params);
                        draw_entities_and_ui(canvas, state, cam, assets, text, sprite_debug, hover_tile);
                    }
                    ApplyScope::FullScreen => {
                        draw_tiles(canvas, state, cam, assets);
                        draw_entities_and_ui(canvas, state, cam, assets, text, sprite_debug, hover_tile);
                        post_process::apply_dither(canvas, params);
                    }
                }
            }
        }
        PostProcessMode::Moebius => {
            if let Some(params) = moebius_params {
                match scope {
                    ApplyScope::TilesOnly => {
                        draw_tiles(canvas, state, cam, assets);
                        post_process::apply_moebius(canvas, params);
                        draw_entities_and_ui(canvas, state, cam, assets, text, sprite_debug, hover_tile);
                    }
                    ApplyScope::FullScreen => {
                        draw_tiles(canvas, state, cam, assets);
                        draw_entities_and_ui(canvas, state, cam, assets, text, sprite_debug, hover_tile);
                        post_process::apply_moebius(canvas, params);
                    }
                }
            }
        }
    }
}
