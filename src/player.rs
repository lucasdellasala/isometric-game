use crate::iso::grid_to_screen;
use crate::tilemap::{TileKind, Tilemap};

// How fast the visual position catches up to the grid position (0.0 = frozen, 1.0 = instant)
const LERP_SPEED: f64 = 0.2;

pub struct Player {
    // Logical position (which tile the player is on)
    pub grid_x: i32,
    pub grid_y: i32,
    // Visual position (where the player is drawn, in screen pixels)
    pub visual_x: f64,
    pub visual_y: f64,
}

impl Player {
    pub fn new(grid_x: i32, grid_y: i32) -> Player {
        let (sx, sy) = grid_to_screen(grid_x, grid_y);
        Player {
            grid_x,
            grid_y,
            visual_x: sx as f64,
            visual_y: sy as f64,
        }
    }

    /// Try to move the player by (dx, dy). Only moves if the target tile is walkable.
    pub fn try_move(&mut self, dx: i32, dy: i32, tilemap: &Tilemap) {
        let new_x = self.grid_x + dx;
        let new_y = self.grid_y + dy;

        if new_x < 0 || new_x >= tilemap.cols || new_y < 0 || new_y >= tilemap.rows {
            return;
        }

        let tile = tilemap.get(new_x, new_y);
        match tile {
            TileKind::Wall | TileKind::Water => return,
            _ => {}
        }

        self.grid_x = new_x;
        self.grid_y = new_y;
    }

    /// Smoothly interpolate visual position toward the grid position.
    /// Called every tick.
    pub fn update(&mut self) {
        let (target_x, target_y) = grid_to_screen(self.grid_x, self.grid_y);
        let tx = target_x as f64;
        let ty = target_y as f64;

        // Linear interpolation: move visual position toward target
        self.visual_x += (tx - self.visual_x) * LERP_SPEED;
        self.visual_y += (ty - self.visual_y) * LERP_SPEED;

        // Snap when close enough to avoid floating point drift
        if (tx - self.visual_x).abs() < 0.5 {
            self.visual_x = tx;
        }
        if (ty - self.visual_y).abs() < 0.5 {
            self.visual_y = ty;
        }
    }
}
