// Isometric projection math: convert between grid coords and screen coords

pub const TILE_WIDTH: i32 = 128;
pub const TILE_HEIGHT: i32 = 64;

/// Convert grid coordinates (col, row) to screen pixel coordinates (x, y).
/// Returns the top-center point of the diamond tile.
pub fn grid_to_screen(grid_x: i32, grid_y: i32) -> (i32, i32) {
    let screen_x = (grid_x - grid_y) * (TILE_WIDTH / 2);
    let screen_y = (grid_x + grid_y) * (TILE_HEIGHT / 2);
    (screen_x, screen_y)
}

/// Convert screen pixel coordinates back to grid coordinates.
/// Useful for mouse clicks: "which tile did the player click?"
pub fn screen_to_grid(screen_x: i32, screen_y: i32) -> (i32, i32) {
    let grid_x = (screen_x / (TILE_WIDTH / 2) + screen_y / (TILE_HEIGHT / 2)) / 2;
    let grid_y = (screen_y / (TILE_HEIGHT / 2) - screen_x / (TILE_WIDTH / 2)) / 2;
    (grid_x, grid_y)
}
