use rand::Rng;

use crate::config;
use crate::render::iso::grid_to_screen;
use crate::core::pathfinding::{self, Pos};
use crate::core::tilemap::Tilemap;

/// NPC visual variant — determines which sprite set to use.
/// Naming: {ethnicity}_{clothes_color}_{hair_color}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NpcVariant {
    AfricanCrBk,
    AfricanGnCr,
    CaucasianGnBn,
    CaucasianYlBk,
    LatinoBkBn,
    LatinoYlBk,
}

impl NpcVariant {
    /// Asset key prefix for this variant (without direction suffix).
    pub fn asset_key(&self) -> &'static str {
        match self {
            NpcVariant::AfricanCrBk => "npc_african_cr_bk",
            NpcVariant::AfricanGnCr => "npc_african_gn_cr",
            NpcVariant::CaucasianGnBn => "npc_caucasian_gn_bn",
            NpcVariant::CaucasianYlBk => "npc_caucasian_yl_bk",
            NpcVariant::LatinoBkBn => "npc_latino_bk_bn",
            NpcVariant::LatinoYlBk => "npc_latino_yl_bk",
        }
    }

    /// Pick a random variant.
    pub fn random() -> NpcVariant {
        let variants = [
            NpcVariant::AfricanCrBk, NpcVariant::AfricanGnCr,
            NpcVariant::CaucasianGnBn, NpcVariant::CaucasianYlBk,
            NpcVariant::LatinoBkBn, NpcVariant::LatinoYlBk,
        ];
        let idx = rand::thread_rng().gen_range(0..variants.len());
        variants[idx]
    }
}

/// Enemy visual type — determines which sprite set to use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyType {
    Orc,
}

impl EnemyType {
    /// Asset key prefix for this enemy type (without direction suffix).
    pub fn asset_key(&self) -> &'static str {
        match self {
            EnemyType::Orc => "enemy_orc",
        }
    }
}

/// 8 cardinal/intercardinal directions as seen on screen (not grid).
/// N = top of screen, S = bottom, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    /// Suffix for sprite filenames (e.g., "S", "NE", "W").
    pub fn sprite_suffix(&self) -> &'static str {
        match self {
            Direction::N  => "N",
            Direction::NE => "NE",
            Direction::E  => "E",
            Direction::SE => "SE",
            Direction::S  => "S",
            Direction::SW => "SW",
            Direction::W  => "W",
            Direction::NW => "NW",
        }
    }

    /// Index into NPC spritesheet frames (0-7).
    /// Frame order in the 1024×256 sheet: S, SW, W, NW, N, NE, E, SE
    pub fn spritesheet_frame(&self) -> u32 {
        match self {
            Direction::S  => 0,
            Direction::SW => 1,
            Direction::W  => 2,
            Direction::NW => 3,
            Direction::N  => 4,
            Direction::NE => 5,
            Direction::E  => 6,
            Direction::SE => 7,
        }
    }

    /// All 8 directions for random selection.
    pub fn all() -> [Direction; 8] {
        [Direction::N, Direction::NE, Direction::E, Direction::SE,
         Direction::S, Direction::SW, Direction::W, Direction::NW]
    }
}

/// Map grid movement (dx, dy) to the screen direction in isometric view.
/// In iso projection, grid axes are rotated 45° from screen axes:
///   grid (0, -1) = screen NW (up-left)       → W key
///   grid (1, -1) = screen N  (up)             → W+D keys
///   grid (1,  0) = screen NE (up-right)       → D key
///   grid (1,  1) = screen E  (right)          → D+S keys (not yet supported)
///   grid (0,  1) = screen SE (down-right)     → S key  (but actually SW visually... wait)
///
/// CORRECTION for iso: each grid cardinal maps to a screen diagonal:
///   W key  → grid (0,-1)  → screen goes up-left    → NW
///   D key  → grid (1, 0)  → screen goes down-right → SE
///   S key  → grid (0, 1)  → screen goes down-left  → SW
///   A key  → grid (-1,0)  → screen goes up-right   → NE
///   W+D    → grid (1,-1)  → screen goes up         → N
///   D+S    → grid (1, 1)  → screen goes right       → E
///   S+A    → grid (-1,1)  → screen goes down        → S
///   A+W    → grid (-1,-1) → screen goes left         → W
fn grid_dir_to_facing(dx: i32, dy: i32) -> Direction {
    match (dx, dy) {
        ( 0, -1) => Direction::NE,  // W key → up-right on screen
        ( 1, -1) => Direction::E,   // W+D → right on screen
        ( 1,  0) => Direction::SE,  // D key → down-right on screen
        ( 1,  1) => Direction::S,   // D+S → straight down
        ( 0,  1) => Direction::SW,  // S key → down-left on screen
        (-1,  1) => Direction::W,   // S+A → left on screen
        (-1,  0) => Direction::NW,  // A key → up-left on screen
        (-1, -1) => Direction::N,   // A+W → straight up
        _ => Direction::SW,
    }
}

/// What kind of entity this is. Determines behavior and rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityKind {
    Player,
    Npc,
    Enemy,
}

/// A game entity: player, NPC, or enemy. Has a position, visual interpolation,
/// and optional pathfinding state.
pub struct Entity {
    pub id: u64,
    pub kind: EntityKind,
    pub name: String,
    // Logical position (which tile the entity is on)
    pub grid_x: i32,
    pub grid_y: i32,
    // Visual position (where the entity is drawn, in screen pixels)
    pub visual_x: f64,
    pub visual_y: f64,
    // Facing direction (screen cardinal). Determines which directional sprite to draw.
    pub facing: Direction,
    // Walk animation state
    pub anim_tick: u32,       // ticks since animation started
    pub anim_moving: bool,    // true while visually in motion (lerp not finished)
    // NPC-specific state
    pub npc_variant: Option<NpcVariant>,  // which sprite set to use (None for non-NPCs)
    pub enemy_type: Option<EnemyType>,    // which enemy sprite set (None for non-enemies)
    idle_rotate_timer: u32,               // ticks until next random facing change
    // Pathfinding state
    path: Vec<Pos>,
    path_index: usize,
    path_timer: u32,
    // Movement cooldown for WASD-style input
    pub move_timer: u32,
}

impl Entity {
    pub fn new(id: u64, kind: EntityKind, name: &str, grid_x: i32, grid_y: i32) -> Entity {
        let (sx, sy) = grid_to_screen(grid_x, grid_y);
        Entity {
            id,
            kind,
            name: String::from(name),
            grid_x,
            grid_y,
            visual_x: sx as f64,
            visual_y: sy as f64,
            facing: Direction::S,
            anim_tick: 0,
            anim_moving: false,
            npc_variant: if kind == EntityKind::Npc { Some(NpcVariant::random()) } else { None },
            enemy_type: if kind == EntityKind::Enemy { Some(EnemyType::Orc) } else { None },
            idle_rotate_timer: if kind == EntityKind::Npc || kind == EntityKind::Enemy {
                rand::thread_rng().gen_range(config::IDLE_ROTATE_MIN_TICKS..=config::IDLE_ROTATE_MAX_TICKS)
            } else { 0 },
            path: vec![],
            path_index: 0,
            path_timer: 0,
            move_timer: 0,
        }
    }

    /// Try to move one tile in a direction. Cancels any active path.
    pub fn try_move(&mut self, dx: i32, dy: i32, tilemap: &Tilemap, blocked: &std::collections::HashSet<(i32, i32)>) {
        self.clear_path();

        // Always update facing, even if movement is blocked (like in Fallout)
        self.facing = grid_dir_to_facing(dx, dy);

        let new_x = self.grid_x + dx;
        let new_y = self.grid_y + dy;

        if new_x < 0 || new_x >= tilemap.cols || new_y < 0 || new_y >= tilemap.rows {
            return;
        }

        if !tilemap.get(new_x, new_y).is_walkable() || blocked.contains(&(new_x, new_y)) {
            return;
        }

        self.grid_x = new_x;
        self.grid_y = new_y;
    }

    /// Set a pathfinding target. Calculates A* and starts walking.
    /// Returns true if a path was found.
    pub fn move_to(&mut self, target_x: i32, target_y: i32, tilemap: &Tilemap, blocked: &std::collections::HashSet<(i32, i32)>) -> bool {
        let start = Pos { x: self.grid_x, y: self.grid_y };
        let goal = Pos { x: target_x, y: target_y };

        match pathfinding::find_path(start, goal, tilemap, blocked) {
            Some(path) => {
                self.path = path;
                self.path_index = 0;
                if self.path_timer == 0 {
                    self.path_timer = config::PATH_STEP_TICKS;
                }
                true
            }
            None => {
                self.clear_path();
                false
            }
        }
    }

    pub fn is_walking(&self) -> bool {
        self.path_index < self.path.len()
    }

    pub fn clear_path(&mut self) {
        self.path.clear();
        self.path_index = 0;
    }

    /// Current walk animation frame index (0..7), or None if idle.
    pub fn walk_frame(&self) -> Option<u32> {
        if self.anim_moving {
            Some((self.anim_tick / config::TICKS_PER_ANIM_FRAME) % config::WALK_ANIM_FRAMES)
        } else {
            None
        }
    }

    /// Advance pathfinding and smooth visual interpolation. Called every tick.
    pub fn update(&mut self) {
        // Follow path if active
        if self.is_walking() {
            if self.path_timer > 0 {
                self.path_timer -= 1;
            } else {
                let next = self.path[self.path_index];
                let dx = next.x - self.grid_x;
                let dy = next.y - self.grid_y;
                self.facing = grid_dir_to_facing(dx, dy);
                self.grid_x = next.x;
                self.grid_y = next.y;
                self.path_index += 1;
                self.path_timer = config::PATH_STEP_TICKS;
            }
        }

        // Smooth visual interpolation
        let (target_x, target_y) = grid_to_screen(self.grid_x, self.grid_y);
        let tx = target_x as f64;
        let ty = target_y as f64;

        self.visual_x += (tx - self.visual_x) * config::LERP_SPEED;
        self.visual_y += (ty - self.visual_y) * config::LERP_SPEED;

        if (tx - self.visual_x).abs() < 0.5 {
            self.visual_x = tx;
        }
        if (ty - self.visual_y).abs() < 0.5 {
            self.visual_y = ty;
        }

        // Walk animation: advance while visually moving, reset when stopped
        let still_moving = (tx - self.visual_x).abs() > 0.5
            || (ty - self.visual_y).abs() > 0.5;
        if still_moving {
            self.anim_moving = true;
            self.anim_tick += 1;
        } else {
            self.anim_moving = false;
            self.anim_tick = 0;
        }

        // Tick down move cooldown
        if self.move_timer > 0 {
            self.move_timer -= 1;
        }

        // NPC idle rotation: randomly change facing every few seconds
        if self.kind == EntityKind::Npc || self.kind == EntityKind::Enemy {
            if self.idle_rotate_timer > 0 {
                self.idle_rotate_timer -= 1;
            } else {
                let mut rng = rand::thread_rng();
                let dirs = Direction::all();
                self.facing = dirs[rng.gen_range(0..dirs.len())];
                self.idle_rotate_timer = rng.gen_range(config::IDLE_ROTATE_MIN_TICKS..=config::IDLE_ROTATE_MAX_TICKS);
            }
        }
    }

    /// Make this entity face toward a grid position (used for NPC interaction).
    pub fn face_toward(&mut self, target_x: i32, target_y: i32) {
        let dx = (target_x - self.grid_x).signum();
        let dy = (target_y - self.grid_y).signum();
        if dx != 0 || dy != 0 {
            self.facing = grid_dir_to_facing(dx, dy);
        }
    }
}
