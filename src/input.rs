/// Actions the player can take. In multiplayer, these are sent from client to server.
/// The GameState processes these without knowing about SDL2 or rendering.
pub enum GameInput {
    /// WASD movement: move one tile in a direction
    MoveDirection { entity_id: u64, dx: i32, dy: i32 },
    /// Click-to-move: pathfind to a target tile
    MoveTo { entity_id: u64, target_x: i32, target_y: i32 },
}

/// Things that happened in the game. In multiplayer, these are sent from server to clients.
/// The renderer and audio system react to these.
pub enum GameEvent {
    /// An entity moved to a new grid position
    EntityMoved { entity_id: u64, grid_x: i32, grid_y: i32 },
    /// A pathfinding request found no valid path
    PathNotFound { entity_id: u64 },
}
