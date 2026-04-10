use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::render::assets::AssetManager;
use crate::render::camera::Camera;
use crate::render::post_process::{self, ApplyScope, DitherParams, MoebiusParams, PostProcessMode};
use crate::core::entity::{Entity, EntityKind};
use crate::ui::debug_menu::DebugMenu;
use crate::core::game_state::GameState;
use crate::render::iso::{grid_to_screen, TILE_HEIGHT, TILE_WIDTH};
use crate::render::text::TextRenderer;
use crate::core::tilemap::TileKind;
use crate::render::decorations;
use crate::config;

/// Convert grid position to screen position with camera offset and zoom applied.
fn to_screen(grid_x: i32, grid_y: i32, cam: &Camera, screen_w: i32, screen_h: i32, zoom: f64) -> (i32, i32) {
    let (sx, sy) = grid_to_screen(grid_x, grid_y);
    let x = ((sx - cam.x) as f64 * zoom) as i32 + screen_w / 2;
    let y = ((sy - cam.y) as f64 * zoom) as i32 + screen_h / 2;
    (x, y)
}

/// Check if a screen position is visible (within the window + margin).
fn is_on_screen(cx: i32, cy: i32, screen_w: i32, screen_h: i32) -> bool {
    cx > -config::CULL_MARGIN - TILE_WIDTH
        && cx < screen_w + config::CULL_MARGIN + TILE_WIDTH
        && cy > -config::CULL_MARGIN - TILE_HEIGHT * 2
        && cy < screen_h + config::CULL_MARGIN + TILE_HEIGHT
}

/// Draw a filled diamond overlay on a tile position (for debug visualization).
fn draw_tile_overlay(canvas: &mut Canvas<Window>, cx: i32, cy: i32, zoom: f64, color: Color) {
    let hw = (TILE_WIDTH as f64 * zoom / 2.0) as i32;
    let hh = (TILE_HEIGHT as f64 * zoom / 2.0) as i32;

    canvas.set_draw_color(color);
    for row in 0..hh * 2 {
        let w = if row < hh {
            hw * row / hh
        } else {
            hw * (hh * 2 - row) / hh
        };
        let _ = canvas.draw_line(
            Point::new(cx - w, cy + row),
            Point::new(cx + w, cy + row),
        );
    }
}

/// Draw a hover highlight (diamond outline) on the tile under the mouse cursor.
fn draw_hover(canvas: &mut Canvas<Window>, grid_x: i32, grid_y: i32, cam: &Camera, sw: i32, sh: i32, zoom: f64) {
    let (cx, cy) = to_screen(grid_x, grid_y, cam, sw, sh, zoom);
    let hw = (TILE_WIDTH as f64 * zoom / 2.0) as i32;
    let hh = (TILE_HEIGHT as f64 * zoom / 2.0) as i32;
    let mid_y = cy + hh;

    canvas.set_draw_color(config::HOVER_COLOR);
    let _ = canvas.draw_line(Point::new(cx, cy), Point::new(cx + hw, mid_y));
    let _ = canvas.draw_line(Point::new(cx + hw, mid_y), Point::new(cx, cy + hh * 2));
    let _ = canvas.draw_line(Point::new(cx, cy + hh * 2), Point::new(cx - hw, mid_y));
    let _ = canvas.draw_line(Point::new(cx - hw, mid_y), Point::new(cx, cy));
}

/// Draw a target marker (small yellow diamond) on a tile.
fn draw_marker(canvas: &mut Canvas<Window>, grid_x: i32, grid_y: i32, cam: &Camera, sw: i32, sh: i32, zoom: f64) {
    let (cx, cy) = to_screen(grid_x, grid_y, cam, sw, sh, zoom);
    let size = (config::MARKER_SIZE_BASE * zoom) as i32;
    let center_y = cy + (TILE_HEIGHT as f64 * zoom) as i32 / 2;

    canvas.set_draw_color(config::MARKER_COLOR);
    for y in 0..size {
        let w = if y < size / 2 { y } else { size - y };
        let _ = canvas.draw_line(
            Point::new(cx - w, center_y - size / 2 + y),
            Point::new(cx + w, center_y - size / 2 + y),
        );
    }
}

/// Deterministic noise value for a grid position. Returns 0-99.
/// Uses multiple hash layers for a more natural distribution.
fn noise(col: i32, row: i32, seed: u32) -> u32 {
    let h1 = (col as u32).wrapping_mul(7919).wrapping_add((row as u32).wrapping_mul(6271));
    let h2 = h1.wrapping_mul(2654435761); // Knuth multiplicative hash
    let h3 = h2.wrapping_add(seed).wrapping_mul(1103515245).wrapping_add(12345);
    (h3 >> 16) % 100
}

/// Get the texture key for a tile kind, with weighted random variant per position.
/// Grass: 60% variant 2, 30% variant 1, 10% variant 3
/// Dirt:  60% variant 2, 30% variant 1, 10% variant 3
/// Water: uses water_variant parameter (1-18)
fn tile_texture_key(kind: TileKind, col: i32, row: i32, water_variant: u32) -> String {
    match kind {
        TileKind::Grass => {
            let n = noise(col, row, 0);
            let variant = if n < 60 { 2 } else if n < 90 { 1 } else { 3 };
            format!("tile_grass_{variant}")
        }
        TileKind::Dirt => {
            let n = noise(col, row, 42);
            let variant = if n < 60 { 2 } else if n < 90 { 1 } else { 3 };
            format!("tile_dirt_{variant}")
        }
        TileKind::Stone => {
            let n = noise(col, row, 99);
            let variant = if n < 60 { 2 } else if n < 90 { 1 } else { 3 };
            format!("tile_stone_{variant}")
        }
        TileKind::Water => {
            format!("tile_water_{water_variant}")
        }
    }
}

use crate::core::entity::Direction;

/// Get the texture key for an entity based on kind, facing, and walk state.
/// Get the texture key for an entity based on its kind, facing direction, and walk state.
/// All entity types use individual PNGs per direction.
/// Walk key format: "{base}_walk_{DIR}_{frame}". Falls back to idle if walk sprite missing.
fn entity_texture_key(entity: &Entity, assets: &AssetManager) -> String {
    let dir = entity.facing.sprite_suffix();

    // Determine the base key and walk key prefix
    let (idle_key, walk_prefix) = match entity.kind {
        EntityKind::Player => (
            format!("entity_player_{}", dir),
            Some(format!("entity_player_walk_{}", dir)),
        ),
        EntityKind::Npc => {
            if let Some(variant) = entity.npc_variant {
                (
                    format!("{}_{}", variant.asset_key(), dir),
                    Some(format!("{}_walk_{}", variant.asset_key(), dir)),
                )
            } else {
                (String::from("entity_npc"), None)
            }
        }
        EntityKind::Enemy => {
            if let Some(etype) = entity.enemy_type {
                (
                    format!("{}_{}", etype.asset_key(), dir),
                    Some(format!("{}_walk_{}", etype.asset_key(), dir)),
                )
            } else {
                (format!("enemy_orc_{}", dir), None)
            }
        }
    };

    // If walking and walk sprites exist, use them. Otherwise fallback to idle.
    if let Some(frame) = entity.walk_frame() {
        if let Some(prefix) = walk_prefix {
            let walk_key = format!("{}_{}", prefix, frame);
            // Check if the walk sprite is loaded; if not, fall back to idle
            if assets.has_texture(&walk_key) {
                return walk_key;
            }
        }
    }

    idle_key
}

/// Draw a tile texture at a screen position, darkened by FOV brightness, scaled by zoom.
/// Always draws at TILE_WIDTH × TILE_HEIGHT regardless of the sprite's actual pixel size.
fn draw_tile(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    key: &str,
    cx: i32,
    cy: i32,
    brightness: f64,
    zoom: f64,
    tile_offset: (i32, i32),
) {
    if let Some(texture) = assets.get_mut(key) {
        let w = (TILE_WIDTH as f64 * zoom) as u32;
        let h = (TILE_HEIGHT as f64 * zoom) as u32;
        let dst = Rect::new(
            cx - w as i32 / 2 + tile_offset.0,
            cy + tile_offset.1,
            w,
            h,
        );

        let b = (brightness * 255.0) as u8;
        texture.set_color_mod(b, b, b);
        let _ = canvas.copy(texture, None, dst);
    }
}

/// Draw an entity sprite at its visual position.
/// If player_rect is Some, sprites that intersect it are drawn semi-transparent.
/// Returns the entity's screen Rect if it's the Player (for use by later sprites).
/// Context for entity interaction highlighting.
struct HighlightCtx {
    /// Player grid position (for adjacency check)
    player_x: i32,
    player_y: i32,
    /// Tile under mouse cursor
    hover_tile: Option<(i32, i32)>,
}

/// Get the highlight color for an entity based on its kind.
fn entity_highlight_color(kind: EntityKind) -> Color {
    match kind {
        EntityKind::Npc => config::HIGHLIGHT_COLOR_NPC,
        EntityKind::Enemy => config::HIGHLIGHT_COLOR_ENEMY,
        EntityKind::Player => Color::RGB(255, 255, 255),
    }
}

fn draw_entity(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    entity: &Entity,
    fov_map: &crate::core::fov::FovMap,
    cam: &Camera,
    sw: i32,
    sh: i32,
    zoom: f64,
    debug_menu: &DebugMenu,
    text: &mut TextRenderer,
    player_rect: Option<Rect>,
    player_depth: i32,
    highlight: &HighlightCtx,
) -> Option<Rect> {
    // Don't draw entities in unexplored or dark areas
    let brightness = fov_map.get_brightness(entity.grid_x, entity.grid_y);
    if brightness < 0.5 {
        return None;
    }

    let cx = ((entity.visual_x as i32 - cam.x) as f64 * zoom) as i32 + sw / 2;
    let cy = ((entity.visual_y as i32 - cam.y) as f64 * zoom) as i32 + sh / 2;

    let key = entity_texture_key(entity, assets);

    // Draw shadow beneath entity (scale from debug menu for live tuning)
    if let Some(shadow) = assets.get_mut("entity_shadow") {
        let shadow_q = shadow.query();
        let shadow_zoom = zoom * debug_menu.shadow_scale;
        let sw2 = (shadow_q.width as f64 * shadow_zoom) as u32;
        let sh2 = (shadow_q.height as f64 * shadow_zoom) as u32;
        let th_shadow = (TILE_HEIGHT as f64 * zoom) as i32;
        let sy = (debug_menu.shadow_offset_y as f64 * zoom) as i32;
        let shadow_dst = Rect::new(
            cx - sw2 as i32 / 2,
            cy + th_shadow / 2 - sh2 as i32 / 2 + sy,
            sw2,
            sh2,
        );
        let b = (brightness * 255.0) as u8;
        shadow.set_color_mod(b, b, b);
        let _ = canvas.copy(shadow, None, shadow_dst);
    }

    if let Some(texture) = assets.get_mut(&key) {
        // Per-entity scale from debug menu (live tuning)
        let type_mult = match entity.kind {
            EntityKind::Player => debug_menu.scale_player,
            EntityKind::Npc => debug_menu.scale_npc,
            EntityKind::Enemy => match entity.enemy_type {
                Some(crate::core::entity::EnemyType::Orc) => debug_menu.scale_enemy_orc,
                _ => 1.0,
            },
        };
        let entity_zoom = zoom * debug_menu.entity_base_scale * type_mult;
        let th = (TILE_HEIGHT as f64 * zoom) as i32;

        let query = texture.query();
        let w = (query.width as f64 * entity_zoom) as u32;
        let h = (query.height as f64 * entity_zoom) as u32;

        // Sprite offset applies to all entities equally (same centering in PNGs)
        let (dx, dy) = debug_menu.get_sprite_offset(entity.facing);
        let off_x = (dx as f64 * zoom) as i32;
        let off_y = (dy as f64 * zoom) as i32;

        let dst = Rect::new(
            cx - w as i32 / 2 + off_x,
            cy + th / 2 - h as i32 + off_y,
            w,
            h,
        );

        // If this entity is in front of the player and overlaps, draw semi-transparent
        // Make entity semi-transparent if it's directly in front of the player.
        // Only applies to entities within 1 tile (Chebyshev distance) AND with a higher
        // depth row (or same tile). Entities further away stay opaque.
        //
        // NOTE: We tried pixel-level rect intersection (full rect, bottom half, bottom third)
        // to only transparentize when sprites actually overlap visually, but the tall sprite
        // rects (128x256) caused false positives on diagonal-back tiles. For entities of
        // size=1 tile, depth-row + proximity check is simpler and visually correct. If we add
        // larger entities or need precision, revisit with per-pixel collision or tighter
        // bounding boxes. See IDEAS.md #2 for context.
        if entity.kind != EntityKind::Player {
            let edx = (entity.grid_x - highlight.player_x).abs();
            let edy = (entity.grid_y - highlight.player_y).abs();
            let is_nearby = edx <= 1 && edy <= 1;
            let entity_depth = entity_depth_row(entity);
            let same_tile = edx == 0 && edy == 0;
            if is_nearby && (same_tile || entity_depth > player_depth) {
                texture.set_alpha_mod(128);
            } else {
                texture.set_alpha_mod(255);
            }
        }

        let _ = canvas.copy(texture, None, dst);
        texture.set_alpha_mod(255); // reset for next use

        // Return player rect for occlusion checks
        let result = if entity.kind == EntityKind::Player { Some(dst) } else { None };

        // Interaction highlight for non-player entities
        if entity.kind != EntityKind::Player {
            // Chebyshev distance <= 1: all 8 surrounding tiles + same tile
            let dx = (entity.grid_x - highlight.player_x).abs();
            let dy = (entity.grid_y - highlight.player_y).abs();
            let is_adjacent = dx <= 1 && dy <= 1;
            let is_hovered = highlight.hover_tile == Some((entity.grid_x, entity.grid_y));

            if is_adjacent || is_hovered {
                let color = entity_highlight_color(entity.kind);
                let px = config::HIGHLIGHT_OUTLINE_PX;

                // Use pre-computed outline points for uniform color.
                // The outline key is "{asset_key}_{frame_index}".
                // Outline key: for NPCs/enemies it's "{base_key}_{direction_index}"
                let outline_key = match entity.kind {
                    EntityKind::Npc => {
                        if let Some(variant) = entity.npc_variant {
                            format!("{}_{}", variant.asset_key(), entity.facing.spritesheet_frame())
                        } else { key.clone() }
                    }
                    EntityKind::Enemy => {
                        let etype_key = entity.enemy_type.map(|e| e.asset_key()).unwrap_or("enemy_orc");
                        format!("{}_{}", etype_key, entity.facing.spritesheet_frame())
                    }
                    _ => key.clone(),
                };

                if let Some(points) = assets.get_outline(&outline_key) {
                    // Scale must match the sprite's actual rendered size (base × type multiplier)
                    let outline_type_mult = match entity.kind {
                        EntityKind::Player => debug_menu.scale_player,
                        EntityKind::Npc => debug_menu.scale_npc,
                        EntityKind::Enemy => match entity.enemy_type {
                            Some(crate::core::entity::EnemyType::Orc) => debug_menu.scale_enemy_orc,
                            _ => 1.0,
                        },
                    };
                    let entity_zoom = zoom * debug_menu.entity_base_scale * outline_type_mult;

                    canvas.set_draw_color(color);
                    for &(ox, oy) in points {
                        let screen_x = dst.x() + (ox as f64 * entity_zoom) as i32;
                        let screen_y = dst.y() + (oy as f64 * entity_zoom) as i32;
                        // Draw a filled square of HIGHLIGHT_OUTLINE_PX size for thickness
                        let _ = canvas.fill_rect(Rect::new(
                            screen_x - px / 2,
                            screen_y - px / 2,
                            px as u32,
                            px as u32,
                        ));
                    }
                }

                // Show action prompt above the entity
                let prompt = if is_adjacent { "[E] Hablar" } else { &entity.name };
                if let Some(tex) = text.render(prompt, config::HIGHLIGHT_PROMPT_FONT_SIZE, color) {
                    let q = tex.query();
                    let label_dst = Rect::new(
                        cx - q.width as i32 / 2,
                        dst.y() - q.height as i32 - config::HIGHLIGHT_PROMPT_GAP,
                        q.width,
                        q.height,
                    );
                    let _ = canvas.copy(tex, None, label_dst);
                }
            }
        }

        // Show sprite key label above player when debug is active
        if debug_menu.visible && entity.kind == EntityKind::Player {
            let (raw_x, raw_y) = debug_menu.get_sprite_offset(entity.facing);
            let label = format!("{} | offset({},{})", key, raw_x, raw_y);
            if let Some(tex) = text.render(&label, 14, Color::RGB(255, 255, 0)) {
                let q = tex.query();
                let label_dst = Rect::new(
                    cx - q.width as i32 / 2,
                    dst.y() - q.height as i32 - 4,
                    q.width,
                    q.height,
                );
                let _ = canvas.copy(tex, None, label_dst);
            }

            // Show mode indicator
            let mode_label = if debug_menu.sprite_per_dir {
                format!("[TAB] Mode: per-direction ({})", entity.facing.sprite_suffix())
            } else {
                "[TAB] Mode: base offset".to_string()
            };
            if let Some(tex) = text.render(&mode_label, 12, Color::RGB(200, 200, 200)) {
                let q = tex.query();
                let mode_dst = Rect::new(
                    cx - q.width as i32 / 2,
                    dst.y() - q.height as i32 - 22,
                    q.width,
                    q.height,
                );
                let _ = canvas.copy(tex, None, mode_dst);
            }
        }

        return result;
    }
    None
}

/// Pre-compute the player's screen Rect without drawing anything.
/// Used to pass to draw_tiles so wall cubes can check occlusion.
fn compute_player_rect(state: &GameState, cam: &Camera, sw: i32, sh: i32, zoom: f64, debug_menu: &DebugMenu, assets: &mut AssetManager) -> Option<Rect> {
    let player = state.local_player()?;
    let cx = ((player.visual_x as i32 - cam.x) as f64 * zoom) as i32 + sw / 2;
    let cy = ((player.visual_y as i32 - cam.y) as f64 * zoom) as i32 + sh / 2;

    let key = entity_texture_key(player, assets);
    let texture = assets.get_mut(&key)?;
    let entity_zoom = zoom * debug_menu.entity_base_scale * debug_menu.scale_player;
    let th = (TILE_HEIGHT as f64 * zoom) as i32;

    let query = texture.query();
    let w = (query.width as f64 * entity_zoom) as u32;
    let h = (query.height as f64 * entity_zoom) as u32;

    let (dx, dy) = debug_menu.get_sprite_offset(player.facing);
    let off_x = (dx as f64 * zoom) as i32;
    let off_y = (dy as f64 * zoom) as i32;

    Some(Rect::new(
        cx - w as i32 / 2 + off_x,
        cy + th / 2 - h as i32 + off_y,
        w,
        h,
    ))
}

/// Draw an isometric wall cube at a grid position.
/// height_tiles = how many tile heights tall the wall is.
/// Sides are solid gray, top is the stone tile texture.
fn draw_wall_cube(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    cx: i32,
    cy: i32,
    height_tiles: i32,
    brightness: f64,
    zoom: f64,
    player_rect: Option<Rect>,
) {
    let hw = (TILE_WIDTH as f64 * zoom / 2.0) as i32;
    let hh = (TILE_HEIGHT as f64 * zoom / 2.0) as i32;
    let cube_h = (TILE_HEIGHT as f64 * zoom * height_tiles as f64) as i32;

    let top_y = cy - cube_h;
    let top_center = top_y + hh;

    // Check if this cube overlaps the player → semi-transparent
    let cube_rect = Rect::new(cx - hw, top_y, (hw * 2) as u32, (cube_h + hh * 2) as u32);
    let alpha = match player_rect {
        Some(pr) if cube_rect.has_intersection(pr) => 128u8,
        _ => 255u8,
    };

    // Left face: solid gray, slightly lighter
    let left_b = (brightness * config::WALL_LEFT_BRIGHTNESS).min(1.0);
    let (lr, lg, lb) = config::WALL_LEFT_COLOR;
    let left_color = Color::RGBA((lr as f64 * left_b) as u8, (lg as f64 * left_b) as u8, (lb as f64 * left_b) as u8, alpha);
    canvas.set_draw_color(left_color);
    for h in 0..cube_h {
        let _ = canvas.draw_line(
            Point::new(cx - hw, top_center + h),
            Point::new(cx, top_y + hh * 2 + h),
        );
    }

    // Right face: solid gray, darker
    let right_b = (brightness * config::WALL_RIGHT_BRIGHTNESS).min(1.0);
    let (rr, rg, rb) = config::WALL_RIGHT_COLOR;
    let right_color = Color::RGBA((rr as f64 * right_b) as u8, (rg as f64 * right_b) as u8, (rb as f64 * right_b) as u8, alpha);
    canvas.set_draw_color(right_color);
    for h in 0..cube_h {
        let _ = canvas.draw_line(
            Point::new(cx, top_y + hh * 2 + h),
            Point::new(cx + hw, top_center + h),
        );
    }

    // Top face: stone tile texture
    // Draw top face tile with same alpha as the cube faces
    if let Some(texture) = assets.get_mut("tile_stone_1") {
        texture.set_alpha_mod(alpha);
    }
    draw_tile(canvas, assets, "tile_stone_1", cx, top_y, brightness, zoom, (0, 0));
    if let Some(texture) = assets.get_mut("tile_stone_1") {
        texture.set_alpha_mod(255);
    }
}

/// Compute the isometric depth row for an entity.
/// Uses the max of grid depth and visual depth (ceiling) to avoid flickering:
/// - Moving down-right: grid depth is higher → entity drawn after destination tiles
/// - Moving up-left: visual depth (ceil) stays high → entity stays at old row until arrival
/// Ceiling division ensures the entity isn't drawn before tiles it visually overlaps.
fn entity_depth_row(entity: &Entity) -> i32 {
    let grid_depth = entity.grid_x + entity.grid_y;
    let half_tile = TILE_HEIGHT / 2;
    // Ceiling division: (visual_y + half_tile - 1) / half_tile
    let visual_depth = (entity.visual_y as i32 + half_tile - 1) / half_tile;
    grid_depth.max(visual_depth)
}

/// Draw only the tilemap (ground tiles). No entities.
/// Call this first, then optionally apply dithering, then call draw_entities.
pub fn draw_tiles(canvas: &mut Canvas<Window>, state: &GameState, cam: &Camera, assets: &mut AssetManager, tile_offset: (i32, i32), water_variant: u32, zoom: f64, player_rect: Option<Rect>, player_depth: i32) {
    let (sw, sh) = canvas.output_size().unwrap_or((1280, 900));
    let sw = sw as i32;
    let sh = sh as i32;

    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    let max_depth = state.tilemap.cols + state.tilemap.rows - 2;

    for depth in 0..=max_depth {
        let col_min = (depth - state.tilemap.rows + 1).max(0);
        let col_max = depth.min(state.tilemap.cols - 1);

        for col in col_min..=col_max {
            let row = depth - col;
            let (cx, cy) = to_screen(col, row, cam, sw, sh, zoom);

            if !is_on_screen(cx, cy, sw, sh) {
                continue;
            }

            let dim = state.fov_map.get_brightness(col, row);
            if dim < 0.01 {
                continue;
            }

            let tile = state.tilemap.get(col, row);
            let key = tile_texture_key(tile, col, row, water_variant);
            draw_tile(canvas, assets, &key, cx, cy, dim, zoom, tile_offset);

            // Test wall cube at grid position (1, 0), 2 tiles tall
            if col == 1 && row == 0 {
                // Only apply transparency if the cube is in front of the player
                let cube_depth = col + row;
                let occlusion_rect = if cube_depth >= player_depth { player_rect } else { None };
                draw_wall_cube(canvas, assets, cx, cy, 2, dim, zoom, occlusion_rect);
            }

            // Draw back grass tufts (behind entities, part of tile layer)
            if tile == TileKind::Grass {
                let tufts = decorations::generate_grass_tufts(col, row);
                decorations::draw_grass_tufts(canvas, assets, &tufts, cx, cy, dim, zoom, Some(true), None);
            }
        }
    }
}

/// Draw entities and UI overlays. Call after draw_tiles (and optional dithering).
/// Entities are depth-sorted and drawn with full color.
pub fn draw_entities_and_ui(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    cam: &Camera,
    assets: &mut AssetManager,
    text: &mut TextRenderer,
    zoom: f64,
    debug_menu: &DebugMenu,
    hover_tile: Option<(i32, i32)>,
) {
    let (sw, sh) = canvas.output_size().unwrap_or((1280, 900));
    let sw = sw as i32;
    let sh = sh as i32;

    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    let player_depth = state.local_player()
        .map(|p| entity_depth_row(p))
        .unwrap_or(0);

    // Pre-compute depth row for each entity
    let mut entity_draw_order: Vec<(i32, usize)> = state.entities
        .iter()
        .enumerate()
        .map(|(idx, e)| (entity_depth_row(e), idx))
        .collect();
    entity_draw_order.sort_by_key(|(depth, _)| *depth);

    let mut entity_cursor = 0;
    let max_depth = state.tilemap.cols + state.tilemap.rows - 2;

    // Track the player's screen rect for occlusion transparency
    let mut player_rect: Option<Rect> = None;

    // Build highlight context for NPC interaction hints
    let player = state.local_player();
    let highlight = HighlightCtx {
        player_x: player.map(|p| p.grid_x).unwrap_or(-999),
        player_y: player.map(|p| p.grid_y).unwrap_or(-999),
        hover_tile,
    };

    for depth in 0..=max_depth {
        // Draw entities for this depth row
        while entity_cursor < entity_draw_order.len() {
            let (entity_depth, entity_idx) = entity_draw_order[entity_cursor];
            if entity_depth > depth {
                break;
            }
            if let Some(pr) = draw_entity(canvas, assets, &state.entities[entity_idx], &state.fov_map, cam, sw, sh, zoom, debug_menu, text, player_rect, player_depth, &highlight) {
                player_rect = Some(pr);
            }
            entity_cursor += 1;
        }

        // Draw front grass tufts for tiles in this depth row (in front of entities)
        let col_min = (depth - state.tilemap.rows + 1).max(0);
        let col_max = depth.min(state.tilemap.cols - 1);
        for col in col_min..=col_max {
            let row = depth - col;
            let tile = state.tilemap.get(col, row);
            if tile != TileKind::Grass {
                continue;
            }
            let dim = state.fov_map.get_brightness(col, row);
            if dim < 0.01 {
                continue;
            }
            let (cx, cy) = to_screen(col, row, cam, sw, sh, zoom);
            if !is_on_screen(cx, cy, sw, sh) {
                continue;
            }
            let tufts = decorations::generate_grass_tufts(col, row);
            decorations::draw_grass_tufts(canvas, assets, &tufts, cx, cy, dim, zoom, Some(false), player_rect);
        }
    }

    // Draw remaining entities
    while entity_cursor < entity_draw_order.len() {
        let (_, entity_idx) = entity_draw_order[entity_cursor];
        if let Some(pr) = draw_entity(canvas, assets, &state.entities[entity_idx], &state.fov_map, cam, sw, sh, zoom, debug_menu, text, player_rect, player_depth, &highlight) {
            player_rect = Some(pr);
        }
        entity_cursor += 1;
    }

    // Draw pathfinding debug overlay if enabled
    if debug_menu.show_pathfinding {
        if let Some(player) = state.local_player() {
            // Compute debug path from player to click target (or hover tile)
            let goal = state.click_target
                .or(hover_tile);
            if let Some((gx, gy)) = goal {
                use crate::core::pathfinding::{self as pf, Pos};
                let start = Pos { x: player.grid_x, y: player.grid_y };
                let goal_pos = Pos { x: gx, y: gy };
                let debug_info = pf::find_path_with_debug(start, goal_pos, &state.tilemap, &state.blocked);

                // Draw closed set (explored tiles) as blue overlay
                for pos in &debug_info.closed_set {
                    let (tcx, tcy) = to_screen(pos.x, pos.y, cam, sw, sh, zoom);
                    if is_on_screen(tcx, tcy, sw, sh) {
                        draw_tile_overlay(canvas, tcx, tcy, zoom, config::PATH_DEBUG_CLOSED_COLOR);
                    }
                }

                // Draw final path as green overlay
                for pos in &debug_info.path {
                    let (tcx, tcy) = to_screen(pos.x, pos.y, cam, sw, sh, zoom);
                    if is_on_screen(tcx, tcy, sw, sh) {
                        draw_tile_overlay(canvas, tcx, tcy, zoom, config::PATH_DEBUG_PATH_COLOR);
                    }
                }

                // Draw start and goal
                let (sx, sy) = to_screen(debug_info.start.x, debug_info.start.y, cam, sw, sh, zoom);
                draw_tile_overlay(canvas, sx, sy, zoom, config::PATH_DEBUG_START_COLOR);
                let (gsx, gsy) = to_screen(debug_info.goal.x, debug_info.goal.y, cam, sw, sh, zoom);
                draw_tile_overlay(canvas, gsx, gsy, zoom, config::PATH_DEBUG_GOAL_COLOR);
            }
        }
    }

    // Draw hover highlight on tile under mouse
    if let Some((hx, hy)) = hover_tile {
        draw_hover(canvas, hx, hy, cam, sw, sh, zoom);
    }

    // Draw click target marker
    if let Some((tx, ty)) = state.click_target {
        if state.fov_map.get_brightness(tx, ty) > 0.5 {
            draw_marker(canvas, tx, ty, cam, sw, sh, zoom);
        }
    }

    // Draw speech bubble above NPC if dialogue is active
    if let Some(dialogue) = &state.active_dialogue {
        // Find the target NPC's screen position
        if let Some(target) = state.get_entity(dialogue.target_id) {
            let tcx = ((target.visual_x as i32 - cam.x) as f64 * zoom) as i32 + sw / 2;
            let tcy = ((target.visual_y as i32 - cam.y) as f64 * zoom) as i32 + sh / 2;
            let entity_zoom = zoom * config::ENTITY_SCALE;
            // Top of entity sprite (approximate)
            let sprite_top = tcy + (TILE_HEIGHT as f64 * zoom) as i32 / 2 - (256.0 * entity_zoom) as i32;

            draw_speech_bubble(
                canvas, text,
                &dialogue.target_name, &dialogue.text,
                tcx, sprite_top,
            );
        }
    }
}

/// Word-wrap a string to fit within max_width pixels at a given font size.
/// Returns a Vec of lines. Like CSS word-wrap: break-word.
fn wrap_text(text_renderer: &mut TextRenderer, content: &str, font_size: u32, max_width: i32) -> Vec<String> {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in &words {
        let test = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{current_line} {word}")
        };

        // Measure width by rendering (cached, so cheap)
        let fits = text_renderer.render(&test, font_size, Color::RGB(255, 255, 255))
            .map(|t| t.query().width as i32 <= max_width)
            .unwrap_or(true);

        if fits {
            current_line = test;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Draw a filled rounded rectangle by scanning rows.
/// Each row's width is adjusted at the corners using a circle equation.
fn fill_rounded_rect(canvas: &mut Canvas<Window>, rect: Rect, radius: i32, color: Color) {
    canvas.set_draw_color(color);

    let x = rect.x();
    let y = rect.y();
    let w = rect.width() as i32;
    let h = rect.height() as i32;
    let r = radius.min(w / 2).min(h / 2);

    for row in 0..h {
        // Calculate horizontal inset for rounded corners
        let inset = if row < r {
            // Top corners
            r - ((r * r - (r - row) * (r - row)) as f64).sqrt() as i32
        } else if row >= h - r {
            // Bottom corners
            let dy = row - (h - r);
            r - ((r * r - dy * dy) as f64).sqrt() as i32
        } else {
            0
        };

        let _ = canvas.draw_line(
            Point::new(x + inset, y + row),
            Point::new(x + w - 1 - inset, y + row),
        );
    }
}

/// Draw a rounded rectangle border by tracing the edge pixels.
fn draw_rounded_rect_border(canvas: &mut Canvas<Window>, rect: Rect, radius: i32, color: Color) {
    canvas.set_draw_color(color);

    let x = rect.x();
    let y = rect.y();
    let w = rect.width() as i32;
    let h = rect.height() as i32;
    let r = radius.min(w / 2).min(h / 2);

    for row in 0..h {
        let inset = if row < r {
            r - ((r * r - (r - row) * (r - row)) as f64).sqrt() as i32
        } else if row >= h - r {
            let dy = row - (h - r);
            r - ((r * r - dy * dy) as f64).sqrt() as i32
        } else {
            0
        };

        if row == 0 || row == h - 1 || inset > 0 {
            // Top/bottom edges or corner rows: draw left and right edge pixels
            let _ = canvas.draw_point(Point::new(x + inset, y + row));
            let _ = canvas.draw_point(Point::new(x + w - 1 - inset, y + row));
            // For top and bottom rows, also draw the full horizontal line
            if row == 0 || row == h - 1 {
                let _ = canvas.draw_line(
                    Point::new(x + inset, y + row),
                    Point::new(x + w - 1 - inset, y + row),
                );
            }
        } else {
            // Straight sides: just left and right edge
            let _ = canvas.draw_point(Point::new(x, y + row));
            let _ = canvas.draw_point(Point::new(x + w - 1, y + row));
        }
    }
}

/// Draw a speech bubble above an NPC with rounded corners and an arrow pointing down.
fn draw_speech_bubble(
    canvas: &mut Canvas<Window>,
    text: &mut TextRenderer,
    speaker: &str,
    dialogue_text: &str,
    anchor_x: i32,
    anchor_y: i32,
) {
    let padding = config::BUBBLE_PADDING;
    let max_w = config::BUBBLE_MAX_WIDTH;
    let radius = config::BUBBLE_CORNER_RADIUS;
    let arrow_h = config::BUBBLE_ARROW_HEIGHT;
    let arrow_hw = config::BUBBLE_ARROW_HALF_WIDTH;
    let line_h = config::BUBBLE_LINE_HEIGHT;
    let gap = config::BUBBLE_GAP_ABOVE_ENTITY;

    // Wrap text to fit within bubble
    let text_max_w = max_w - padding * 2;
    let lines = wrap_text(text, dialogue_text, config::BUBBLE_TEXT_FONT_SIZE, text_max_w);
    let hint = "[E] Cerrar";

    // Calculate bubble dimensions
    let name_h = config::BUBBLE_NAME_FONT_SIZE as i32 + 4;
    let text_h = lines.len() as i32 * line_h;
    let hint_h = config::BUBBLE_HINT_FONT_SIZE as i32 + 4;
    let content_h = name_h + text_h + hint_h + padding; // gap between sections
    let bubble_h = content_h + padding * 2;

    // Find the widest line to size the bubble
    let mut bubble_w = 0i32;
    // Check name width
    if let Some(tex) = text.render(speaker, config::BUBBLE_NAME_FONT_SIZE, config::BUBBLE_NAME_COLOR) {
        bubble_w = bubble_w.max(tex.query().width as i32);
    }
    // Check each text line
    for line in &lines {
        if let Some(tex) = text.render(line, config::BUBBLE_TEXT_FONT_SIZE, config::BUBBLE_TEXT_COLOR) {
            bubble_w = bubble_w.max(tex.query().width as i32);
        }
    }
    bubble_w = (bubble_w + padding * 2).min(max_w).max(100);

    // Position: centered above the entity, arrow points to anchor
    let bx = anchor_x - bubble_w / 2;
    let by = anchor_y - bubble_h - arrow_h - gap;

    let bubble_rect = Rect::new(bx, by, bubble_w as u32, bubble_h as u32);

    // Draw filled rounded rectangle
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    fill_rounded_rect(canvas, bubble_rect, radius, config::BUBBLE_BG_COLOR);
    draw_rounded_rect_border(canvas, bubble_rect, radius, config::BUBBLE_BORDER_COLOR);

    // Draw arrow (triangle pointing down to the NPC)
    let arrow_top = by + bubble_h;
    let arrow_bottom = arrow_top + arrow_h;
    canvas.set_draw_color(config::BUBBLE_BG_COLOR);
    for row in 0..arrow_h {
        let half = arrow_hw - (row * arrow_hw / arrow_h);
        let _ = canvas.draw_line(
            Point::new(anchor_x - half, arrow_top + row),
            Point::new(anchor_x + half, arrow_top + row),
        );
    }
    // Arrow border (left and right edges)
    canvas.set_draw_color(config::BUBBLE_BORDER_COLOR);
    let _ = canvas.draw_line(Point::new(anchor_x - arrow_hw, arrow_top), Point::new(anchor_x, arrow_bottom));
    let _ = canvas.draw_line(Point::new(anchor_x + arrow_hw, arrow_top), Point::new(anchor_x, arrow_bottom));

    // Draw speaker name
    let mut cy = by + padding;
    if let Some(tex) = text.render(speaker, config::BUBBLE_NAME_FONT_SIZE, config::BUBBLE_NAME_COLOR) {
        let q = tex.query();
        let dst = Rect::new(bx + padding, cy, q.width, q.height);
        let _ = canvas.copy(tex, None, dst);
    }
    cy += name_h;

    // Draw wrapped text lines
    for line in &lines {
        if let Some(tex) = text.render(line, config::BUBBLE_TEXT_FONT_SIZE, config::BUBBLE_TEXT_COLOR) {
            let q = tex.query();
            let dst = Rect::new(bx + padding, cy, q.width, q.height);
            let _ = canvas.copy(tex, None, dst);
        }
        cy += line_h;
    }

    // Draw hint
    cy += 2;
    if let Some(tex) = text.render(hint, config::BUBBLE_HINT_FONT_SIZE, config::BUBBLE_HINT_COLOR) {
        let q = tex.query();
        let dst = Rect::new(bx + padding, cy, q.width, q.height);
        let _ = canvas.copy(tex, None, dst);
    }
}

/// Orchestrate the full render pass: tiles, optional post-process, entities+UI.
/// Centralizes the render pipeline so main.rs stays thin.
pub fn render_frame(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    cam: &Camera,
    assets: &mut AssetManager,
    text: &mut TextRenderer,
    mode: PostProcessMode,
    scope: ApplyScope,
    dither_params: Option<&DitherParams>,
    moebius_params: Option<&MoebiusParams>,
    debug_menu: &DebugMenu,
    hover_tile: Option<(i32, i32)>,
    tile_offset: (i32, i32),
    water_variant: u32,
) {
    let zoom = debug_menu.camera_zoom;
    let (sw, sh) = canvas.output_size().unwrap_or((1280, 900));
    let player_rect = compute_player_rect(state, cam, sw as i32, sh as i32, zoom, debug_menu, assets);
    let player_depth = state.local_player()
        .map(|p| entity_depth_row(p))
        .unwrap_or(0);
    match mode {
        PostProcessMode::Off => {
            draw_tiles(canvas, state, cam, assets, tile_offset, water_variant, zoom, player_rect, player_depth);
            draw_entities_and_ui(canvas, state, cam, assets, text, zoom, debug_menu, hover_tile);
        }
        PostProcessMode::Dithering => {
            if let Some(params) = dither_params {
                match scope {
                    ApplyScope::TilesOnly => {
                        draw_tiles(canvas, state, cam, assets, tile_offset, water_variant, zoom, player_rect, player_depth);
                        post_process::apply_dither(canvas, params);
                        draw_entities_and_ui(canvas, state, cam, assets, text, zoom, debug_menu, hover_tile);
                    }
                    ApplyScope::FullScreen => {
                        draw_tiles(canvas, state, cam, assets, tile_offset, water_variant, zoom, player_rect, player_depth);
                        draw_entities_and_ui(canvas, state, cam, assets, text, zoom, debug_menu, hover_tile);
                        post_process::apply_dither(canvas, params);
                    }
                }
            }
        }
        PostProcessMode::Moebius => {
            if let Some(params) = moebius_params {
                match scope {
                    ApplyScope::TilesOnly => {
                        draw_tiles(canvas, state, cam, assets, tile_offset, water_variant, zoom, player_rect, player_depth);
                        post_process::apply_moebius(canvas, params);
                        draw_entities_and_ui(canvas, state, cam, assets, text, zoom, debug_menu, hover_tile);
                    }
                    ApplyScope::FullScreen => {
                        draw_tiles(canvas, state, cam, assets, tile_offset, water_variant, zoom, player_rect, player_depth);
                        draw_entities_and_ui(canvas, state, cam, assets, text, zoom, debug_menu, hover_tile);
                        post_process::apply_moebius(canvas, params);
                    }
                }
            }
        }
    }
}
