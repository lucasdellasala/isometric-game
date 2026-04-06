use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::assets::AssetManager;
use crate::camera::{Camera, CAMERA_ZOOM};
use crate::entity::{Entity, EntityKind};
use crate::game_state::GameState;
use crate::iso::{grid_to_screen, TILE_HEIGHT, TILE_WIDTH};
use crate::text::TextRenderer;
#[allow(unused_imports)]
use crate::tilemap::TileKind;

const CULL_MARGIN: i32 = 64;

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
        TileKind::Stone => "tile_grass",
        TileKind::Water => "tile_water",
    }
}

/// Get the texture key for an entity kind.
fn entity_texture_key(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Player => "entity_player",
        EntityKind::Npc => "entity_npc",
        EntityKind::Enemy => "entity_enemy",
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
    fov_map: &crate::fov::FovMap,
    cam: &Camera,
    sw: i32,
    sh: i32,
) {
    // Don't draw entities in unexplored or dark areas
    let brightness = fov_map.get_brightness(entity.grid_x, entity.grid_y);
    if brightness < 0.5 {
        return;
    }

    let cx = ((entity.visual_x as i32 - cam.x) as f64 * CAMERA_ZOOM) as i32 + sw / 2;
    let cy = ((entity.visual_y as i32 - cam.y) as f64 * CAMERA_ZOOM) as i32 + sh / 4;

    let key = entity_texture_key(entity.kind);

    if let Some(texture) = assets.get_mut(key) {
        let query = texture.query();
        let w = (query.width as f64 * CAMERA_ZOOM) as u32;
        let h = (query.height as f64 * CAMERA_ZOOM) as u32;
        let th = (TILE_HEIGHT as f64 * CAMERA_ZOOM) as i32;
        // Center the sprite on the tile, raise it above the ground
        let dst = Rect::new(
            cx - w as i32 / 2,
            cy + th / 2 - h as i32,
            w,
            h,
        );
        let _ = canvas.copy(texture, None, dst);
    }
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

/// Draw the entire game world. Reads GameState immutably.
/// Uses row-by-row interleaving: for each isometric row, draw tiles first,
/// then entities whose visual depth falls in that row. This makes entities
/// correctly appear behind walls and other tall tiles.
pub fn draw_world(canvas: &mut Canvas<Window>, state: &GameState, cam: &Camera, assets: &mut AssetManager, text: &mut TextRenderer) {
    let (sw, sh) = canvas.output_size().unwrap_or((1280, 900));
    let sw = sw as i32;
    let sh = sh as i32;

    // Enable alpha blending for FOV overlay
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    // Pre-compute depth row for each entity so we can draw them at the right time.
    // depth_row = col + row in isometric space (the "diagonal" that determines draw order).
    let mut entity_draw_order: Vec<(i32, usize)> = state.entities
        .iter()
        .enumerate()
        .map(|(idx, e)| (entity_depth_row(e), idx))
        .collect();
    // Sort by depth row so we can iterate through them efficiently
    entity_draw_order.sort_by_key(|(depth, _)| *depth);

    let mut entity_cursor = 0;

    // In isometric view, the draw order goes by "depth rows" where depth = col + row.
    // For a map of size (cols, rows), depth ranges from 0 to (cols-1 + rows-1).
    let max_depth = state.tilemap.cols + state.tilemap.rows - 2;

    for depth in 0..=max_depth {
        // Draw all tiles in this depth row (where col + row == depth).
        // col ranges from max(0, depth - rows + 1) to min(depth, cols - 1).
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
        }

        // Draw entities whose depth row matches this depth
        while entity_cursor < entity_draw_order.len() {
            let (entity_depth, entity_idx) = entity_draw_order[entity_cursor];
            if entity_depth > depth {
                break;
            }
            draw_entity(canvas, assets, &state.entities[entity_idx], &state.fov_map, cam, sw, sh);
            entity_cursor += 1;
        }
    }

    // Draw any remaining entities (shouldn't happen, but safety net)
    while entity_cursor < entity_draw_order.len() {
        let (_, entity_idx) = entity_draw_order[entity_cursor];
        draw_entity(canvas, assets, &state.entities[entity_idx], &state.fov_map, cam, sw, sh);
        entity_cursor += 1;
    }

    // Draw click target marker (on top of everything)
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
