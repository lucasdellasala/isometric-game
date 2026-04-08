use std::collections::HashSet;

use crate::core::entity::{Entity, EntityKind};
use crate::core::fov::FovMap;
use crate::core::input::{GameEvent, GameInput};
use crate::config;
use crate::core::tilemap::Tilemap;

/// Active dialogue state — which entity is talking and what they say.
pub struct ActiveDialogue {
    pub target_id: u64,
    pub target_name: String,
    pub text: String,
}

/// All game state lives here. Pure logic — no SDL2, no rendering, no audio.
/// In multiplayer, the server owns this. Clients send GameInput, receive GameEvent.
pub struct GameState {
    pub tilemap: Tilemap,
    pub entities: Vec<Entity>,
    pub fov_map: FovMap,
    pub click_target: Option<(i32, i32)>,
    pub local_player_id: u64,
    pub active_dialogue: Option<ActiveDialogue>,
    /// FOV radius in tiles. Determines how far the player can see.
    /// Will be per-entity in the future (e.g., elf > human).
    pub fov_radius: i32,
    /// Grid positions blocked by objects (walls, props). Entities can't walk here.
    pub blocked: HashSet<(i32, i32)>,
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
            active_dialogue: None,
            fov_radius: config::DEFAULT_FOV_RADIUS,
            blocked: HashSet::new(),
            next_entity_id: 0,
        };

        // Block the test wall cube position
        state.blocked.insert((1, 0));

        // Spawn the player at the center of the map
        let center_x = state.tilemap.cols / 2;
        let center_y = state.tilemap.rows / 2;
        let player_id = state.spawn_entity(EntityKind::Player, "Player", center_x, center_y);
        state.local_player_id = player_id;

        // Spawn entities defined in the map file
        for spawn in &state.tilemap.entity_spawns.clone() {
            let kind = match spawn.kind.as_str() {
                "Npc" => EntityKind::Npc,
                "Enemy" => EntityKind::Enemy,
                "Player" => continue, // Skip — player is always spawned at (0,0) for now
                other => {
                    eprintln!("Unknown entity kind in map: {other}");
                    continue;
                }
            };
            state.spawn_entity(kind, &spawn.name, spawn.x, spawn.y);
        }

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
                    let blocked = &self.blocked;
                    if let Some(entity) = self.entities.iter_mut().find(|e| e.id == entity_id) {
                        entity.try_move(dx, dy, tilemap, blocked);
                        entity.move_timer = config::MOVE_COOLDOWN;
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
                let blocked = &self.blocked;
                if let Some(entity) = self.entities.iter_mut().find(|e| e.id == entity_id) {
                    let found = entity.move_to(target_x, target_y, tilemap, blocked);
                    if found {
                        self.click_target = Some((target_x, target_y));
                    } else {
                        events.push(GameEvent::PathNotFound { entity_id });
                    }
                }
            }
            GameInput::Interact { entity_id } => {
                // Find the player's position
                let player_pos = self.get_entity(entity_id)
                    .map(|e| (e.grid_x, e.grid_y));

                if let Some((px, py)) = player_pos {
                    // Look for an adjacent NPC/Enemy (4 cardinal directions)
                    let adjacent: Vec<(i32, i32)> = vec![
                        (px - 1, py), (px + 1, py),
                        (px, py - 1), (px, py + 1),
                    ];

                    let target = self.entities.iter().find(|e| {
                        e.id != entity_id
                            && adjacent.contains(&(e.grid_x, e.grid_y))
                    });

                    if let Some(target) = target {
                        let target_id = target.id;
                        let target_name = target.name.clone();

                        // Make NPC face the player
                        if let Some(npc) = self.entities.iter_mut().find(|e| e.id == target_id) {
                            npc.face_toward(px, py);
                        }

                        events.push(GameEvent::InteractionStarted {
                            entity_id,
                            target_id,
                        });
                        self.active_dialogue = Some(ActiveDialogue {
                            target_id,
                            target_name: target_name.clone(),
                            text: format!("Hola, soy {}. ¡Bienvenido, aventurero!", target_name),
                        });
                    } else {
                        events.push(GameEvent::NothingToInteract);
                    }
                }
            }
            GameInput::DismissDialogue => {
                self.active_dialogue = None;
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
            self.fov_map.compute(player.grid_x, player.grid_y, self.fov_radius, &self.tilemap);
        }

        events
    }
}
