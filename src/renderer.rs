use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::assets::AssetManager;
use crate::camera::Camera;
use crate::entity::{Entity, EntityKind};
use crate::game_state::GameState;
use crate::iso::{grid_to_screen, TILE_HEIGHT, TILE_WIDTH};
use crate::tilemap::TileKind;

const CULL_MARGIN: i32 = 64;

/// Convert grid position to screen position with camera offset applied.
fn to_screen(grid_x: i32, grid_y: i32, cam: &Camera, screen_w: i32, screen_h: i32) -> (i32, i32) {
    let (sx, sy) = grid_to_screen(grid_x, grid_y);
    (sx - cam.x + screen_w / 2, sy - cam.y + screen_h / 4)
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
    let size = 6;
    let center_y = cy + TILE_HEIGHT / 2;

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
        TileKind::Water => "tile_water",
        TileKind::Wall => "tile_wall_top",
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

/// Draw a tile texture at a screen position, darkened by FOV brightness.
fn draw_tile(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    key: &str,
    cx: i32,
    cy: i32,
    brightness: f64,
) {
    if let Some(texture) = assets.get_mut(key) {
        let query = texture.query();
        let dst = Rect::new(
            cx - query.width as i32 / 2,
            cy,
            query.width,
            query.height,
        );

        // Darken texture using color mod (only affects non-transparent pixels)
        let b = (brightness * 255.0) as u8;
        texture.set_color_mod(b, b, b);
        let _ = canvas.copy(texture, None, dst);
    }
}

/// Darken a color by brightness factor.
fn darken(color: Color, brightness: f64) -> Color {
    Color::RGB(
        (color.r as f64 * brightness) as u8,
        (color.g as f64 * brightness) as u8,
        (color.b as f64 * brightness) as u8,
    )
}

/// Draw the left side face of a wall using lines (parallelogram shape).
fn fill_left_face(canvas: &mut Canvas<Window>, cx: i32, cy: i32, height: i32, color: Color) {
    let half_w = TILE_WIDTH / 2;
    let half_h = TILE_HEIGHT / 2;

    canvas.set_draw_color(color);
    for h in 0..height {
        let _ = canvas.draw_line(
            Point::new(cx - half_w, cy + half_h + h),
            Point::new(cx, cy + TILE_HEIGHT + h),
        );
    }
}

/// Draw the right side face of a wall using lines (parallelogram shape).
fn fill_right_face(canvas: &mut Canvas<Window>, cx: i32, cy: i32, height: i32, color: Color) {
    let half_w = TILE_WIDTH / 2;
    let half_h = TILE_HEIGHT / 2;

    canvas.set_draw_color(color);
    for h in 0..height {
        let _ = canvas.draw_line(
            Point::new(cx, cy + TILE_HEIGHT + h),
            Point::new(cx + half_w, cy + half_h + h),
        );
    }
}

/// Draw a wall tile. If a full wall sprite exists (64x64), draws it as one image.
/// Otherwise falls back to line-drawn side faces + textured top.
fn draw_wall(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    cx: i32,
    cy: i32,
    wall_height: i32,
    brightness: f64,
) {
    // Try full wall sprite (includes top + side face in one image)
    if assets.get_mut("tile_wall").is_some() {
        // Wall sprite is taller than a flat tile — draw it offset upward
        if let Some(texture) = assets.get_mut("tile_wall") {
            let query = texture.query();
            let b = (brightness * 255.0) as u8;
            texture.set_color_mod(b, b, b);
            let dst = Rect::new(
                cx - query.width as i32 / 2,
                cy - (query.height as i32 - TILE_HEIGHT),
                query.width,
                query.height,
            );
            let _ = canvas.copy(texture, None, dst);
        }
    } else {
        // Fallback: line-drawn side faces + placeholder top
        fill_left_face(canvas, cx, cy - wall_height, wall_height, darken(Color::RGB(120, 120, 120), brightness));
        fill_right_face(canvas, cx, cy - wall_height, wall_height, darken(Color::RGB(90, 90, 90), brightness));
        draw_tile(canvas, assets, "tile_wall_top", cx, cy - wall_height, brightness);
    }
}

/// Draw an entity sprite at its visual position.
fn draw_entity(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    entity: &Entity,
    cam: &Camera,
    sw: i32,
    sh: i32,
) {
    let cx = entity.visual_x as i32 - cam.x + sw / 2;
    let cy = entity.visual_y as i32 - cam.y + sh / 4;

    let key = entity_texture_key(entity.kind);

    if let Some(texture) = assets.get_mut(key) {
        let query = texture.query();
        // Center the sprite on the tile, raise it above the ground
        let dst = Rect::new(
            cx - query.width as i32 / 2,
            cy + TILE_HEIGHT / 2 - query.height as i32,
            query.width,
            query.height,
        );
        let _ = canvas.copy(texture, None, dst);
    }
}

/// Draw the entire game world. Reads GameState immutably.
pub fn draw_world(canvas: &mut Canvas<Window>, state: &GameState, cam: &Camera, assets: &mut AssetManager) {
    let (sw, sh) = canvas.output_size().unwrap_or((1280, 900));
    let sw = sw as i32;
    let sh = sh as i32;

    // Enable alpha blending for FOV overlay
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    // Draw all tiles (back to front)
    for row in 0..state.tilemap.rows {
        for col in 0..state.tilemap.cols {
            let (cx, cy) = to_screen(col, row, cam, sw, sh);

            if !is_on_screen(cx, cy, sw, sh) {
                continue;
            }

            let dim = state.fov_map.get_brightness(col, row);

            if dim < 0.01 {
                continue;
            }

            let tile = state.tilemap.get(col, row);
            let height = tile.height();

            if height > 0 {
                draw_wall(canvas, assets, cx, cy, height, dim);
            } else {
                let key = tile_texture_key(tile);
                draw_tile(canvas, assets, key, cx, cy, dim);
            }
        }
    }

    // Draw click target marker
    if let Some((tx, ty)) = state.click_target {
        if state.fov_map.get_brightness(tx, ty) > 0.5 {
            draw_marker(canvas, tx, ty, cam, sw, sh);
        }
    }

    // Draw all entities on top
    for entity in &state.entities {
        draw_entity(canvas, assets, entity, cam, sw, sh);
    }
}
