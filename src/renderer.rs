use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::camera::Camera;
use crate::iso::{grid_to_screen, TILE_HEIGHT, TILE_WIDTH};
use crate::player::Player;
use crate::tilemap::Tilemap;

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;
const SCREEN_CENTER_X: i32 = SCREEN_WIDTH / 2;
const SCREEN_CENTER_Y: i32 = SCREEN_HEIGHT / 4;

// Margin in pixels outside the screen to still draw tiles
// (prevents popping at edges, especially for tall tiles)
const CULL_MARGIN: i32 = 64;

/// Convert grid position to screen position with camera offset applied.
fn to_screen(grid_x: i32, grid_y: i32, cam: &Camera) -> (i32, i32) {
    let (sx, sy) = grid_to_screen(grid_x, grid_y);
    (sx - cam.x + SCREEN_CENTER_X, sy - cam.y + SCREEN_CENTER_Y)
}

/// Draw a filled diamond (the flat top face of a tile).
fn fill_diamond(canvas: &mut Canvas<Window>, cx: i32, cy: i32, color: Color) {
    let half_w = TILE_WIDTH / 2;
    let half_h = TILE_HEIGHT / 2;

    // Fill the diamond line by line (scanline fill)
    // Top half: from top point down to the middle
    for y_offset in 0..half_h {
        let width_at_y = (y_offset * half_w) / half_h;
        let draw_y = cy + y_offset;
        canvas.set_draw_color(color);
        let _ = canvas.draw_line(
            Point::new(cx - width_at_y, draw_y),
            Point::new(cx + width_at_y, draw_y),
        );
    }
    // Bottom half: from middle down to the bottom point
    for y_offset in 0..half_h {
        let width_at_y = half_w - (y_offset * half_w) / half_h;
        let draw_y = cy + half_h + y_offset;
        canvas.set_draw_color(color);
        let _ = canvas.draw_line(
            Point::new(cx - width_at_y, draw_y),
            Point::new(cx + width_at_y, draw_y),
        );
    }
}

/// Draw the left side face of a tall tile (for the 3D effect).
fn fill_left_face(canvas: &mut Canvas<Window>, cx: i32, cy: i32, height: i32, color: Color) {
    let half_w = TILE_WIDTH / 2;
    let half_h = TILE_HEIGHT / 2;

    // Left face: from bottom-left edge of diamond going down by `height`
    canvas.set_draw_color(color);
    for h in 0..height {
        let _ = canvas.draw_line(
            Point::new(cx - half_w, cy + half_h + h),
            Point::new(cx, cy + TILE_HEIGHT + h),
        );
    }
}

/// Draw the right side face of a tall tile (for the 3D effect).
fn fill_right_face(canvas: &mut Canvas<Window>, cx: i32, cy: i32, height: i32, color: Color) {
    let half_w = TILE_WIDTH / 2;
    let half_h = TILE_HEIGHT / 2;

    // Right face: from bottom-right edge of diamond going down by `height`
    // Slightly darker than left face for depth
    let darker = Color::RGB(
        color.r.saturating_sub(30),
        color.g.saturating_sub(30),
        color.b.saturating_sub(30),
    );
    canvas.set_draw_color(darker);
    for h in 0..height {
        let _ = canvas.draw_line(
            Point::new(cx, cy + TILE_HEIGHT + h),
            Point::new(cx + half_w, cy + half_h + h),
        );
    }
}

/// Draw the player as a colored diamond on the tile, raised slightly.
fn draw_player(canvas: &mut Canvas<Window>, player: &Player, cam: &Camera) {
    // Use visual position (smooth) instead of grid position (snappy)
    let cx = player.visual_x as i32 - cam.x + SCREEN_CENTER_X;
    let cy = player.visual_y as i32 - cam.y + SCREEN_CENTER_Y;

    // Player body: a smaller diamond on top of the tile, raised up
    let body_height = 20;
    let body_half_w = TILE_WIDTH / 4;
    let body_half_h = TILE_HEIGHT / 4;

    // Base position: center of the tile, raised by body_height
    let base_y = cy + TILE_HEIGHT / 2 - body_height;

    // Draw body (filled rectangle-ish shape using lines)
    let body_color = Color::RGB(200, 60, 60);
    canvas.set_draw_color(body_color);
    for y in 0..body_height {
        let _ = canvas.draw_line(
            Point::new(cx - body_half_w / 2, base_y + y),
            Point::new(cx + body_half_w / 2, base_y + y),
        );
    }

    // Draw head (small diamond on top)
    let head_color = Color::RGB(240, 200, 150);
    let head_size = 6;
    for y in 0..head_size {
        let w = if y < head_size / 2 { y } else { head_size - y };
        canvas.set_draw_color(head_color);
        let _ = canvas.draw_line(
            Point::new(cx - w, base_y - head_size + y),
            Point::new(cx + w, base_y - head_size + y),
        );
    }

    // Shadow on the tile (small dark diamond)
    let shadow_color = Color::RGBA(0, 0, 0, 80);
    canvas.set_draw_color(shadow_color);
    for y in 0..body_half_h {
        let w = if y < body_half_h / 2 {
            (y * body_half_w) / body_half_h
        } else {
            ((body_half_h - y) * body_half_w) / body_half_h
        };
        let _ = canvas.draw_line(
            Point::new(cx - w, cy + TILE_HEIGHT / 2 - body_half_h / 2 + y),
            Point::new(cx + w, cy + TILE_HEIGHT / 2 - body_half_h / 2 + y),
        );
    }
}

/// Check if a screen position is visible (within the window + margin).
fn is_visible(cx: i32, cy: i32) -> bool {
    cx > -CULL_MARGIN - TILE_WIDTH
        && cx < SCREEN_WIDTH + CULL_MARGIN + TILE_WIDTH
        && cy > -CULL_MARGIN - TILE_HEIGHT * 2
        && cy < SCREEN_HEIGHT + CULL_MARGIN + TILE_HEIGHT
}

/// Draw a target marker (small yellow diamond) on a tile.
fn draw_marker(canvas: &mut Canvas<Window>, grid_x: i32, grid_y: i32, cam: &Camera) {
    let (cx, cy) = to_screen(grid_x, grid_y, cam);
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

/// Draw the entire tilemap and player with correct depth sorting (back to front).
pub fn draw_world(canvas: &mut Canvas<Window>, tilemap: &Tilemap, player: &Player, cam: &Camera, click_target: Option<(i32, i32)>) {
    // Draw all tiles first (back to front)
    for row in 0..tilemap.rows {
        for col in 0..tilemap.cols {
            let (cx, cy) = to_screen(col, row, cam);

            // Frustum culling: skip tiles outside the screen
            if !is_visible(cx, cy) {
                continue;
            }

            let tile = tilemap.get(col, row);
            let height = tile.height();

            if height > 0 {
                fill_left_face(canvas, cx, cy - height, height, tile.side_color());
                fill_right_face(canvas, cx, cy - height, height, tile.side_color());
                fill_diamond(canvas, cx, cy - height, tile.top_color());
            } else {
                fill_diamond(canvas, cx, cy, tile.top_color());
            }
        }
    }

    // Draw click target marker
    if let Some((tx, ty)) = click_target {
        draw_marker(canvas, tx, ty, cam);
    }

    // Draw player on top of all tiles
    draw_player(canvas, player, cam);
}
