use crate::tilemap::{TileKind, Tilemap};

pub struct Player {
    pub grid_x: i32,
    pub grid_y: i32,
}

impl Player {
    pub fn new(grid_x: i32, grid_y: i32) -> Player {
        Player { grid_x, grid_y }
    }

    /// Try to move the player by (dx, dy). Only moves if the target tile is walkable.
    pub fn try_move(&mut self, dx: i32, dy: i32, tilemap: &Tilemap) {
        let new_x = self.grid_x + dx;
        let new_y = self.grid_y + dy;

        // Check bounds
        if new_x < 0 || new_x >= tilemap.cols || new_y < 0 || new_y >= tilemap.rows {
            return;
        }

        // Check if walkable (walls and water block movement)
        let tile = tilemap.get(new_x, new_y);
        match tile {
            TileKind::Wall | TileKind::Water => return,
            _ => {}
        }

        self.grid_x = new_x;
        self.grid_y = new_y;
    }
}
