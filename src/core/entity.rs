use crate::render::iso::grid_to_screen;
use crate::core::pathfinding::{self, Pos};
use crate::core::tilemap::Tilemap;

const LERP_SPEED: f64 = 0.2;
const PATH_STEP_TICKS: u32 = 8;
const WALK_ANIM_FRAMES: u32 = 8;     // total frames in walk cycle
const TICKS_PER_ANIM_FRAME: u32 = 4; // ticks before advancing to next frame

/// Map grid movement (dx, dy) to the sprite angle that looks correct in isometric view.
/// In iso projection, grid axes are rotated 45° from screen axes:
///   grid (0, -1) = screen north     → sprite 180° (back to camera)
///   grid (1,  0) = screen southeast → sprite 270°
///   grid (0,  1) = screen south     → sprite 000° (facing camera)
///   grid (-1, 0) = screen northwest → sprite 090°
fn grid_dir_to_facing(dx: i32, dy: i32) -> u16 {
    match (dx, dy) {
        ( 0, -1) => 180,  // grid north → screen up → back to camera
        ( 1, -1) => 225,  // grid NE → screen right
        ( 1,  0) =>  90,  // grid east → screen SE
        ( 1,  1) => 315,  // grid SE → screen down-right
        ( 0,  1) =>   0,  // grid south → screen down → facing camera
        (-1,  1) =>  45,  // grid SW → screen down-left
        (-1,  0) => 270,  // grid west → screen NW
        (-1, -1) => 135,  // grid NW → screen up-left
        _ => 0,
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
    // Facing direction in degrees (0, 45, 90, ..., 315).
    // Determines which directional sprite to draw.
    pub facing: u16,
    // Walk animation state
    pub anim_tick: u32,       // ticks since animation started
    pub anim_moving: bool,    // true while visually in motion (lerp not finished)
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
            facing: 0,
            anim_tick: 0,
            anim_moving: false,
            path: vec![],
            path_index: 0,
            path_timer: 0,
            move_timer: 0,
        }
    }

    /// Try to move one tile in a direction. Cancels any active path.
    pub fn try_move(&mut self, dx: i32, dy: i32, tilemap: &Tilemap) {
        self.clear_path();

        // Always update facing, even if movement is blocked (like in Fallout)
        self.facing = grid_dir_to_facing(dx, dy);

        let new_x = self.grid_x + dx;
        let new_y = self.grid_y + dy;

        if new_x < 0 || new_x >= tilemap.cols || new_y < 0 || new_y >= tilemap.rows {
            return;
        }

        if !tilemap.get(new_x, new_y).is_walkable() {
            return;
        }

        self.grid_x = new_x;
        self.grid_y = new_y;
    }

    /// Set a pathfinding target. Calculates A* and starts walking.
    /// Returns true if a path was found.
    pub fn move_to(&mut self, target_x: i32, target_y: i32, tilemap: &Tilemap) -> bool {
        let start = Pos { x: self.grid_x, y: self.grid_y };
        let goal = Pos { x: target_x, y: target_y };

        match pathfinding::find_path(start, goal, tilemap) {
            Some(path) => {
                self.path = path;
                self.path_index = 0;
                if self.path_timer == 0 {
                    self.path_timer = PATH_STEP_TICKS;
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
            Some((self.anim_tick / TICKS_PER_ANIM_FRAME) % WALK_ANIM_FRAMES)
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
    }
}
