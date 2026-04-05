use sdl2::pixels::Color;

/// What type of tile sits on this grid cell
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileKind {
    Grass,
    Dirt,
    Water,
    Wall,  // Tall object — demonstrates depth sorting
}

impl TileKind {
    /// Top face color of the tile
    pub fn top_color(&self) -> Color {
        match self {
            TileKind::Grass => Color::RGB(80, 150, 80),
            TileKind::Dirt => Color::RGB(150, 120, 70),
            TileKind::Water => Color::RGB(60, 100, 180),
            TileKind::Wall => Color::RGB(160, 160, 160),
        }
    }

    /// Whether this tile has height (draws a vertical face)
    pub fn height(&self) -> i32 {
        match self {
            TileKind::Wall => 24,
            _ => 0,
        }
    }

    /// Side face color (darker shade for the 3D effect)
    pub fn side_color(&self) -> Color {
        match self {
            TileKind::Wall => Color::RGB(120, 120, 120),
            _ => Color::RGB(0, 0, 0), // Not used for flat tiles
        }
    }
}

/// The tilemap: a 2D grid of tiles
pub struct Tilemap {
    pub cols: i32,
    pub rows: i32,
    tiles: Vec<TileKind>,
}

impl Tilemap {
    /// Create a test map with some variety
    pub fn new_test(cols: i32, rows: i32) -> Tilemap {
        let size = (cols * rows) as usize;
        let mut tiles = vec![TileKind::Grass; size];

        // Dirt path (diagonal)
        for i in 0..cols.min(rows) {
            tiles[(i * cols + i) as usize] = TileKind::Dirt;
            if i + 1 < cols {
                tiles[(i * cols + i + 1) as usize] = TileKind::Dirt;
            }
        }

        // Water pond
        for row in 3..6 {
            for col in 8..11 {
                if row < rows && col < cols {
                    tiles[(row * cols + col) as usize] = TileKind::Water;
                }
            }
        }

        // Wall structure (L-shape)
        for col in 2..6 {
            if col < cols {
                tiles[(1 * cols + col) as usize] = TileKind::Wall;
            }
        }
        for row in 1..4 {
            if row < rows && 2 < cols {
                tiles[(row * cols + 2) as usize] = TileKind::Wall;
            }
        }

        Tilemap { cols, rows, tiles }
    }

    /// Get the tile at (col, row). Returns Grass if out of bounds.
    pub fn get(&self, col: i32, row: i32) -> TileKind {
        if col < 0 || col >= self.cols || row < 0 || row >= self.rows {
            return TileKind::Grass;
        }
        self.tiles[(row * self.cols + col) as usize]
    }
}
