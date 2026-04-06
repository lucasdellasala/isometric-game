use serde::Deserialize;
use std::fs;

/// What type of tile sits on this grid cell
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum TileKind {
    Grass,
    Dirt,
    Stone,
    Water,
}

/// Which edge of a tile a wall sits on.
/// Only south and east — north/west are the south/east of the adjacent tile.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WallEdge {
    South,
    East,
}

/// A wall object that sits on the edge of a floor tile.
/// Example: { "x": 8, "y": 2, "edge": "south", "variant": "stone" }
#[derive(Debug, Clone, Deserialize)]
pub struct WallObject {
    pub x: i32,
    pub y: i32,
    pub edge: WallEdge,
    #[serde(default = "default_wall_variant")]
    pub variant: String,
}

fn default_wall_variant() -> String {
    String::from("stone")
}

/// An entity spawn point defined in the map JSON.
/// Example: { "kind": "Npc", "name": "Guide", "x": 4, "y": 3 }
#[derive(Debug, Clone, Deserialize)]
pub struct EntitySpawn {
    pub kind: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
}

impl TileKind {
    /// Whether this tile blocks movement
    pub fn is_walkable(&self) -> bool {
        match self {
            TileKind::Water => false,
            _ => true,
        }
    }
}

/// JSON structure for loading a tilemap from file
#[derive(Deserialize)]
struct TilemapFile {
    cols: i32,
    rows: i32,
    tiles: Vec<TileKind>,
    #[serde(default)]
    walls: Vec<WallObject>,
    #[serde(default)]
    entities: Vec<EntitySpawn>,
}

/// The tilemap: a 2D grid of tiles, plus wall objects on tile edges.
pub struct Tilemap {
    pub cols: i32,
    pub rows: i32,
    tiles: Vec<TileKind>,
    /// Wall objects that sit on tile edges (blocking movement across that edge).
    pub walls: Vec<WallObject>,
    /// Entity spawn points loaded from the map file.
    pub entity_spawns: Vec<EntitySpawn>,
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
            walls: data.walls,
            entity_spawns: data.entities,
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
