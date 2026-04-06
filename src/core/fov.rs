use crate::core::tilemap::Tilemap;

// Brightness level for explored-but-not-visible tiles
const EXPLORED_BRIGHTNESS: f64 = 0.35;

/// Fog of war map: tracks visibility with distance-based falloff.
pub struct FovMap {
    pub cols: i32,
    pub rows: i32,
    visible: Vec<bool>,
    explored: Vec<bool>,
    brightness: Vec<f64>,  // Brightness based on distance from player
    radius: i32,
}

impl FovMap {
    pub fn new(cols: i32, rows: i32) -> FovMap {
        let size = (cols * rows) as usize;
        FovMap {
            cols,
            rows,
            visible: vec![false; size],
            explored: vec![false; size],
            brightness: vec![0.0; size],
            radius: 0,
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < self.cols && y >= 0 && y < self.rows {
            Some((y * self.cols + x) as usize)
        } else {
            None
        }
    }

    /// Get the brightness for a tile (0.0 = hidden, up to 1.0 = fully visible).
    pub fn get_brightness(&self, x: i32, y: i32) -> f64 {
        match self.index(x, y) {
            Some(i) => self.brightness[i],
            None => 0.0,
        }
    }

    /// Check if a tile has ever been explored.
    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        match self.index(x, y) {
            Some(i) => self.explored[i],
            None => false,
        }
    }

    fn set_visible(&mut self, x: i32, y: i32, origin_x: i32, origin_y: i32) {
        if let Some(i) = self.index(x, y) {
            self.visible[i] = true;
            self.explored[i] = true;

            // Distance-based falloff: closer to player = brighter
            let dx = (x - origin_x) as f64;
            let dy = (y - origin_y) as f64;
            let dist = (dx * dx + dy * dy).sqrt();
            let max_dist = self.radius as f64;

            // Smooth falloff: full brightness in inner 60%, fade to 0 at edge
            let falloff_start = max_dist * 0.5;
            let brightness = if dist <= falloff_start {
                1.0
            } else {
                let t = (dist - falloff_start) / (max_dist - falloff_start);
                (1.0 - t).max(0.0)
            };

            // Keep the highest brightness (in case multiple octants overlap)
            if brightness > self.brightness[i] {
                self.brightness[i] = brightness;
            }
        }
    }

    fn clear_visible(&mut self) {
        for i in 0..self.visible.len() {
            self.visible[i] = false;
            // Set explored tiles to their dim brightness, hidden to 0
            self.brightness[i] = if self.explored[i] {
                EXPLORED_BRIGHTNESS
            } else {
                0.0
            };
        }
    }

    /// Recalculate FOV from a given position using shadowcasting.
    pub fn compute(&mut self, origin_x: i32, origin_y: i32, radius: i32, tilemap: &Tilemap) {
        self.radius = radius;
        self.clear_visible();

        // The origin is always visible at full brightness
        if let Some(i) = self.index(origin_x, origin_y) {
            self.visible[i] = true;
            self.explored[i] = true;
            self.brightness[i] = 1.0;
        }

        for octant in 0..8 {
            self.cast_light(origin_x, origin_y, radius, 1, 1.0, 0.0, octant, tilemap);
        }
    }

    fn cast_light(
        &mut self,
        ox: i32, oy: i32,
        radius: i32,
        row: i32,
        mut start_slope: f64,
        end_slope: f64,
        octant: u8,
        tilemap: &Tilemap,
    ) {
        if start_slope < end_slope {
            return;
        }

        let mut blocked = false;
        let mut next_start_slope = start_slope;

        for j in row..=radius {
            if blocked {
                return;
            }

            for dx in (-j)..=0 {
                let dy = -j;

                let (map_x, map_y) = transform_octant(ox, oy, dx, dy, octant);

                let left_slope = (dx as f64 - 0.5) / (dy as f64 + 0.5);
                let right_slope = (dx as f64 + 0.5) / (dy as f64 - 0.5);

                if start_slope < right_slope {
                    continue;
                }
                if end_slope > left_slope {
                    break;
                }

                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= radius * radius {
                    self.set_visible(map_x, map_y, ox, oy);
                }

                // TODO: update to check wall objects on tile edges instead of tile type
                // Currently no TileKind blocks vision — only wall objects will (once connected)
                let is_wall = false;

                if blocked {
                    if is_wall {
                        next_start_slope = right_slope;
                    } else {
                        blocked = false;
                        start_slope = next_start_slope;
                    }
                } else if is_wall && j < radius {
                    blocked = true;
                    self.cast_light(ox, oy, radius, j + 1, start_slope, left_slope, octant, tilemap);
                    next_start_slope = right_slope;
                }
            }
        }
    }
}

fn transform_octant(ox: i32, oy: i32, dx: i32, dy: i32, octant: u8) -> (i32, i32) {
    match octant {
        0 => (ox + dx, oy + dy),
        1 => (ox + dy, oy + dx),
        2 => (ox - dy, oy + dx),
        3 => (ox - dx, oy + dy),
        4 => (ox - dx, oy - dy),
        5 => (ox - dy, oy - dx),
        6 => (ox + dy, oy - dx),
        7 => (ox + dx, oy - dy),
        _ => (ox, oy),
    }
}
