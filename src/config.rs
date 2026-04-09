//! Centralized configuration constants for the game.
//! All tunable visual, gameplay, and UI values live here.
//! Never hardcode these values inline — always reference this module.

use sdl2::pixels::Color;

// =============================================================================
// TILE & PROJECTION
// =============================================================================

/// Isometric tile dimensions in pixels.
pub const TILE_WIDTH: i32 = 128;
pub const TILE_HEIGHT: i32 = 64;

// =============================================================================
// CAMERA
// =============================================================================

/// Default camera zoom level. 1.0 = default, 2.0 = twice as close.
pub const DEFAULT_CAMERA_ZOOM: f64 = 1.6;

// =============================================================================
// ENTITY RENDERING
// =============================================================================

/// Manual offset to adjust entity sprite positioning on the tile.
/// Positive X = move sprite right, Positive Y = move sprite down.
pub const ENTITY_OFFSET_X: i32 = 2;
pub const ENTITY_OFFSET_Y: i32 = 30;

/// Scale factor for entity sprites relative to zoom.
/// 1.0 = full size, 0.66 = two thirds.
pub const ENTITY_SCALE: f64 = 0.66;

// =============================================================================
// ENTITY BEHAVIOR
// =============================================================================

/// Visual interpolation speed (0.0–1.0). Higher = snappier movement.
pub const LERP_SPEED: f64 = 0.2;

/// Ticks between each pathfinding step.
pub const PATH_STEP_TICKS: u32 = 8;

/// Total frames in the walk animation cycle.
pub const WALK_ANIM_FRAMES: u32 = 8;

/// Ticks before advancing to the next walk animation frame.
pub const TICKS_PER_ANIM_FRAME: u32 = 4;

/// NPC idle rotation: minimum ticks before random facing change (3 sec @ 60fps).
pub const IDLE_ROTATE_MIN_TICKS: u32 = 180;

/// NPC idle rotation: maximum ticks before random facing change (8 sec @ 60fps).
pub const IDLE_ROTATE_MAX_TICKS: u32 = 480;

/// Movement input cooldown in ticks (WASD).
pub const MOVE_COOLDOWN: u32 = 6;

// =============================================================================
// FOV & VISIBILITY
// =============================================================================

/// Default field-of-view radius in tiles.
pub const DEFAULT_FOV_RADIUS: i32 = 18;

/// Brightness level for explored-but-not-visible tiles.
pub const EXPLORED_BRIGHTNESS: f64 = 0.35;

// =============================================================================
// INTERACTION HIGHLIGHT
// =============================================================================

/// Outline thickness in pixels for entity highlight.
pub const HIGHLIGHT_OUTLINE_PX: i32 = 2;

/// Highlight color for friendly NPCs (green).
pub const HIGHLIGHT_COLOR_NPC: Color = Color::RGB(100, 255, 100);

/// Highlight color for hostile enemies (red).
pub const HIGHLIGHT_COLOR_ENEMY: Color = Color::RGB(255, 60, 60);

/// Alpha for highlight when player is adjacent (interaction range).
pub const HIGHLIGHT_ALPHA_ADJACENT: u8 = 140;

/// Alpha for highlight when mouse hovers over entity.
pub const HIGHLIGHT_ALPHA_HOVER: u8 = 80;

/// Font size for interaction prompt text.
pub const HIGHLIGHT_PROMPT_FONT_SIZE: u32 = 14;

/// Vertical gap between prompt text and entity sprite top.
pub const HIGHLIGHT_PROMPT_GAP: i32 = 6;

// =============================================================================
// HOVER & MARKERS
// =============================================================================

/// Color of the tile hover highlight diamond.
pub const HOVER_COLOR: Color = Color::RGBA(255, 255, 255, 120);

/// Color of the click target marker.
pub const MARKER_COLOR: Color = Color::RGB(255, 255, 0);

/// Size factor for the click target marker (multiplied by zoom).
pub const MARKER_SIZE_BASE: f64 = 6.0;

// =============================================================================
// DIALOGUE BOX
// =============================================================================

/// Dialogue box height in pixels.
pub const DIALOGUE_BOX_HEIGHT: i32 = 120;

/// Dialogue box margin from screen edges.
pub const DIALOGUE_BOX_MARGIN: i32 = 20;

/// Dialogue box inner padding.
pub const DIALOGUE_BOX_PADDING: i32 = 16;

/// Dialogue box background color.
pub const DIALOGUE_BG_COLOR: Color = Color::RGBA(10, 10, 30, 200);

/// Dialogue box border color.
pub const DIALOGUE_BORDER_COLOR: Color = Color::RGB(180, 160, 100);

/// Speaker name font size.
pub const DIALOGUE_NAME_FONT_SIZE: u32 = 22;

/// Speaker name color.
pub const DIALOGUE_NAME_COLOR: Color = Color::RGB(255, 220, 100);

/// Dialogue text font size.
pub const DIALOGUE_TEXT_FONT_SIZE: u32 = 18;

/// Dialogue text color.
pub const DIALOGUE_TEXT_COLOR: Color = Color::RGB(230, 230, 230);

/// Hint text font size.
pub const DIALOGUE_HINT_FONT_SIZE: u32 = 14;

/// Hint text color.
pub const DIALOGUE_HINT_COLOR: Color = Color::RGB(150, 150, 150);

/// Gap between speaker name and dialogue text.
pub const DIALOGUE_TEXT_GAP: i32 = 30;

/// Gap between dialogue box bottom and hint text.
pub const DIALOGUE_HINT_BOTTOM_GAP: i32 = 16;

// =============================================================================
// SPEECH BUBBLE (globo de diálogo above NPC)
// =============================================================================

/// Speech bubble background color.
pub const BUBBLE_BG_COLOR: Color = Color::RGBA(20, 15, 10, 230);

/// Speech bubble border color.
pub const BUBBLE_BORDER_COLOR: Color = Color::RGB(160, 140, 90);

/// Speech bubble text font size.
pub const BUBBLE_TEXT_FONT_SIZE: u32 = 14;

/// Speech bubble text color.
pub const BUBBLE_TEXT_COLOR: Color = Color::RGB(230, 225, 210);

/// Speech bubble speaker name font size.
pub const BUBBLE_NAME_FONT_SIZE: u32 = 13;

/// Speech bubble speaker name color.
pub const BUBBLE_NAME_COLOR: Color = Color::RGB(255, 220, 100);

/// Speech bubble inner padding in pixels.
pub const BUBBLE_PADDING: i32 = 10;

/// Speech bubble corner radius in pixels.
pub const BUBBLE_CORNER_RADIUS: i32 = 8;

/// Speech bubble maximum width in pixels.
pub const BUBBLE_MAX_WIDTH: i32 = 280;

/// Speech bubble arrow height in pixels.
pub const BUBBLE_ARROW_HEIGHT: i32 = 10;

/// Speech bubble arrow half-width in pixels.
pub const BUBBLE_ARROW_HALF_WIDTH: i32 = 8;

/// Gap between speech bubble arrow and entity sprite top.
pub const BUBBLE_GAP_ABOVE_ENTITY: i32 = 4;

/// Line height multiplier for text wrapping.
pub const BUBBLE_LINE_HEIGHT: i32 = 18;

/// Hint text in speech bubble.
pub const BUBBLE_HINT_FONT_SIZE: u32 = 11;

/// Hint text color in speech bubble.
pub const BUBBLE_HINT_COLOR: Color = Color::RGB(140, 135, 120);

// =============================================================================
// PATHFINDING DEBUG OVERLAY
// =============================================================================

/// Color for tiles in the closed set (fully explored).
pub const PATH_DEBUG_CLOSED_COLOR: Color = Color::RGBA(100, 100, 150, 60);

/// Color for the final path.
pub const PATH_DEBUG_PATH_COLOR: Color = Color::RGBA(80, 255, 80, 100);

/// Color for the goal tile.
pub const PATH_DEBUG_GOAL_COLOR: Color = Color::RGBA(255, 255, 0, 120);

/// Color for the start tile.
pub const PATH_DEBUG_START_COLOR: Color = Color::RGBA(0, 150, 255, 120);

// =============================================================================
// FRUSTUM CULLING
// =============================================================================

/// Extra margin around screen for tile frustum culling.
pub const CULL_MARGIN: i32 = TILE_WIDTH * 2;

// =============================================================================
// WALL CUBE (test)
// =============================================================================

/// Left face brightness factor for wall cubes.
pub const WALL_LEFT_BRIGHTNESS: f64 = 0.7;

/// Right face brightness factor for wall cubes.
pub const WALL_RIGHT_BRIGHTNESS: f64 = 0.5;

/// Left face base color for wall cubes.
pub const WALL_LEFT_COLOR: (u8, u8, u8) = (130, 130, 135);

/// Right face base color for wall cubes.
pub const WALL_RIGHT_COLOR: (u8, u8, u8) = (110, 110, 115);
