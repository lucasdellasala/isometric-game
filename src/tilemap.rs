use sdl2::pixels::Color;
use serde::Deserialize;
use std::fs;

/// What type of tile sits on this grid cell
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum TileKind {
    Grass,
    Dirt,
    Water,
    Wall,
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
            _ => Color::RGB(0, 0, 0),
        }
    }
}

/// JSON structure for loading a tilemap from file
#[derive(Deserialize)]
struct TilemapFile {
    cols: i32,
    rows: i32,
    tiles: Vec<TileKind>,
}

/// The tilemap: a 2D grid of tiles
pub struct Tilemap {
    pub cols: i32,
    pub rows: i32,
    tiles: Vec<TileKind>,
}

impl Tilemap {
    /// Load a tilemap from a JSON file.
    pub fn from_file(path: &str) -> Result<Tilemap, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {path}: {e}"))?;

        let data: TilemapFile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {path}: {e}"))?;

        let expected = (data.cols * data.rows) as usize;
        if data.tiles.len() != expected {
            return Err(format!(
                "Map says {}x{} ({expected} tiles) but found {} tiles",
                data.cols, data.rows, data.tiles.len()
            ));
        }

        Ok(Tilemap {
            cols: data.cols,
            rows: data.rows,
            tiles: data.tiles,
        })
    }

    /// Get the tile at (col, row). Returns Grass if out of bounds.
    pub fn get(&self, col: i32, row: i32) -> TileKind {
        if col < 0 || col >= self.cols || row < 0 || row >= self.rows {
            return TileKind::Grass;
        }
        self.tiles[(row * self.cols + col) as usize]
    }
}
