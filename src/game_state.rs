use crate::entity::{Entity, EntityKind};
use crate::fov::FovMap;
use crate::input::{GameEvent, GameInput};
use crate::tilemap::Tilemap;

const MOVE_COOLDOWN: u32 = 6;
const FOV_RADIUS: i32 = 10;

/// All game state lives here. Pure logic — no SDL2, no rendering, no audio.
/// In multiplayer, the server owns this. Clients send GameInput, receive GameEvent.
pub struct GameState {
    pub tilemap: Tilemap,
    pub entities: Vec<Entity>,
    pub fov_map: FovMap,
    pub click_target: Option<(i32, i32)>,
    pub local_player_id: u64,
    next_entity_id: u64,
}

impl GameState {
    pub fn new(tilemap: Tilemap) -> GameState {
        let fov_map = FovMap::new(tilemap.cols, tilemap.rows);

        let mut state = GameState {
            tilemap,
            entities: vec![],
            fov_map,
            click_target: None,
            local_player_id: 0,
            next_entity_id: 0,
        };

        // Spawn the player at (0, 0)
        let player_id = state.spawn_entity(EntityKind::Player, "Player", 0, 0);
        state.local_player_id = player_id;

        // Test NPC to prove multi-entity works
        state.spawn_entity(EntityKind::Npc, "Guide", 4, 3);

        state
    }

    /// Spawn a new entity and return its ID.
    pub fn spawn_entity(&mut self, kind: EntityKind, name: &str, grid_x: i32, grid_y: i32) -> u64 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(Entity::new(id, kind, name, grid_x, grid_y));
        id
    }

    /// Find an entity by ID (immutable).
    pub fn get_entity(&self, id: u64) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    /// Find an entity by ID (mutable).
    fn get_entity_mut(&mut self, id: u64) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    /// Get the local player entity (immutable).
    pub fn local_player(&self) -> Option<&Entity> {
        self.get_entity(self.local_player_id)
    }

    /// Process a game input. In multiplayer, the server calls this for each client's input.
    pub fn apply_input(&mut self, input: GameInput) -> Vec<GameEvent> {
        let mut events = vec![];

        match input {
            GameInput::MoveDirection { entity_id, dx, dy } => {
                // Only move if cooldown is done and not walking a path
                let can_move = self.get_entity(entity_id)
                    .map(|e| e.move_timer == 0 && !e.is_walking())
                    .unwrap_or(false);

                if can_move {
                    // Need to borrow tilemap separately from entity
                    let tilemap = &self.tilemap;
                    if let Some(entity) = self.entities.iter_mut().find(|e| e.id == entity_id) {
                        entity.try_move(dx, dy, tilemap);
                        entity.move_timer = MOVE_COOLDOWN;
                        events.push(GameEvent::EntityMoved {
                            entity_id,
                            grid_x: entity.grid_x,
                            grid_y: entity.grid_y,
                        });
                    }
                }
            }
            GameInput::MoveTo { entity_id, target_x, target_y } => {
                let tilemap = &self.tilemap;
                if let Some(entity) = self.entities.iter_mut().find(|e| e.id == entity_id) {
                    let found = entity.move_to(target_x, target_y, tilemap);
                    if found {
                        self.click_target = Some((target_x, target_y));
                    } else {
                        events.push(GameEvent::PathNotFound { entity_id });
                    }
                }
            }
        }

        events
    }

    /// Advance the game by one tick. Called 60 times per second.
    pub fn tick(&mut self) -> Vec<GameEvent> {
        let events = vec![];

        // Update all entities (pathfinding advancement + visual interpolation)
        for entity in &mut self.entities {
            entity.update();
        }

        // Clear click marker when local player arrives
        if let Some(player) = self.get_entity(self.local_player_id) {
            if !player.is_walking() && self.click_target.is_some() {
                self.click_target = None;
            }
        }

        // Recompute FOV from local player position
        if let Some(player) = self.get_entity(self.local_player_id) {
            self.fov_map.compute(player.grid_x, player.grid_y, FOV_RADIUS, &self.tilemap);
        }

        events
    }
}
