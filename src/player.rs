use crate::iso::grid_to_screen;
use crate::pathfinding::{self, Pos};
use crate::tilemap::{TileKind, Tilemap};

// How fast the visual position catches up to the grid position (0.0 = frozen, 1.0 = instant)
const LERP_SPEED: f64 = 0.2;

// How many ticks to wait before advancing to the next path step
const PATH_STEP_TICKS: u32 = 8;

pub struct Player {
    // Logical position (which tile the player is on)
    pub grid_x: i32,
    pub grid_y: i32,
    // Visual position (where the player is drawn, in screen pixels)
    pub visual_x: f64,
    pub visual_y: f64,
    // Pathfinding: list of positions to walk through
    path: Vec<Pos>,
    path_index: usize,
    path_timer: u32,
}

impl Player {
    pub fn new(grid_x: i32, grid_y: i32) -> Player {
        let (sx, sy) = grid_to_screen(grid_x, grid_y);
        Player {
            grid_x,
            grid_y,
            visual_x: sx as f64,
            visual_y: sy as f64,
            path: vec![],
            path_index: 0,
            path_timer: 0,
        }
    }

    /// Try to move the player by (dx, dy). Only moves if the target tile is walkable.
    /// Cancels any active path.
    pub fn try_move(&mut self, dx: i32, dy: i32, tilemap: &Tilemap) {
        self.clear_path();

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

    /// Set a pathfinding target. Calculates A* and starts walking.
    pub fn move_to(&mut self, target_x: i32, target_y: i32, tilemap: &Tilemap) {
        let start = Pos { x: self.grid_x, y: self.grid_y };
        let goal = Pos { x: target_x, y: target_y };

        match pathfinding::find_path(start, goal, tilemap) {
            Some(path) => {
                self.path = path;
                self.path_index = 0;
                // Keep existing timer to prevent speed exploit from rapid clicking
                if self.path_timer == 0 {
                    self.path_timer = PATH_STEP_TICKS;
                }
            }
            None => {
                // No path found — do nothing
                self.clear_path();
            }
        }
    }

    /// Returns true if the player is currently following a path.
    pub fn is_walking(&self) -> bool {
        self.path_index < self.path.len()
    }

    /// Cancel the current path.
    pub fn clear_path(&mut self) {
        self.path.clear();
        self.path_index = 0;
    }

    /// Advance along the path (called every tick).
    pub fn update(&mut self) {
        // Follow path if active
        if self.is_walking() {
            if self.path_timer > 0 {
                self.path_timer -= 1;
            } else {
                let next = self.path[self.path_index];
                self.grid_x = next.x;
                self.grid_y = next.y;
                self.path_index += 1;
                self.path_timer = PATH_STEP_TICKS;
            }
        }

        // Smooth visual interpolation
        let (target_x, target_y) = grid_to_screen(self.grid_x, self.grid_y);
        let tx = target_x as f64;
        let ty = target_y as f64;

        self.visual_x += (tx - self.visual_x) * LERP_SPEED;
        self.visual_y += (ty - self.visual_y) * LERP_SPEED;

        if (tx - self.visual_x).abs() < 0.5 {
            self.visual_x = tx;
        }
        if (ty - self.visual_y).abs() < 0.5 {
            self.visual_y = ty;
        }
    }
}
