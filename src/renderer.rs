use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::camera::Camera;
use crate::iso::{grid_to_screen, TILE_HEIGHT, TILE_WIDTH};
use crate::tilemap::Tilemap;

const SCREEN_CENTER_X: i32 = 400; // WINDOW_WIDTH / 2
const SCREEN_CENTER_Y: i32 = 150; // WINDOW_HEIGHT / 4

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

/// Draw the entire tilemap with correct depth sorting (back to front).
pub fn draw_tilemap(canvas: &mut Canvas<Window>, tilemap: &Tilemap, cam: &Camera) {
    // Draw back to front: low rows first, then high rows
    // This is the painter's algorithm for isometric rendering
    for row in 0..tilemap.rows {
        for col in 0..tilemap.cols {
            let tile = tilemap.get(col, row);
            let (cx, cy) = to_screen(col, row, cam);
            let height = tile.height();

            if height > 0 {
                // Tall tile: draw side faces first, then top face raised up
                fill_left_face(canvas, cx, cy - height, height, tile.side_color());
                fill_right_face(canvas, cx, cy - height, height, tile.side_color());
                fill_diamond(canvas, cx, cy - height, tile.top_color());
            } else {
                // Flat tile: just the diamond
                fill_diamond(canvas, cx, cy, tile.top_color());
            }
        }
    }
}
