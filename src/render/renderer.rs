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

use crate::core::entity::facing_to_npc_frame;

/// Get the texture key and optional src_rect for an entity.
/// For NPCs with variant spritesheets, returns the sheet key + src_rect for the direction frame.
/// For player/enemy, returns the individual sprite key + None.
fn entity_texture_info(entity: &Entity) -> (String, Option<Rect>) {
    match entity.kind {
        EntityKind::Player => {
            let key = if let Some(frame) = entity.walk_frame() {
                format!("entity_player_walk_{:03}_{}", entity.facing, frame)
            } else {
                format!("entity_player_{:03}", entity.facing)
            };
            (key, None)
        }
        EntityKind::Npc => {
            if let Some(variant) = entity.npc_variant {
                let key = String::from(variant.asset_key());
                let frame = facing_to_npc_frame(entity.facing);
                let src = Rect::new((frame * 128) as i32, 0, 128, 256);
                (key, Some(src))
            } else {
                (String::from("entity_npc"), None)
            }
        }
        EntityKind::Enemy => (String::from("entity_enemy"), None),
    }
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
    highlight: &HighlightCtx,
) -> Option<Rect> {
    // Don't draw entities in unexplored or dark areas
    let brightness = fov_map.get_brightness(entity.grid_x, entity.grid_y);
    if brightness < 0.5 {
        return None;
    }

    let cx = ((entity.visual_x as i32 - cam.x) as f64 * zoom) as i32 + sw / 2;
    let cy = ((entity.visual_y as i32 - cam.y) as f64 * zoom) as i32 + sh / 2;

    let (key, src_rect) = entity_texture_info(entity);

    if let Some(texture) = assets.get_mut(&key) {
        let entity_zoom = zoom * config::ENTITY_SCALE;
        let th = (TILE_HEIGHT as f64 * zoom) as i32;

        // For spritesheet NPCs, use the frame dimensions (128x256), not the full sheet
        let (src_w, src_h) = match src_rect {
            Some(r) => (r.width(), r.height()),
            None => {
                let q = texture.query();
                (q.width, q.height)
            }
        };
        let w = (src_w as f64 * entity_zoom) as u32;
        let h = (src_h as f64 * entity_zoom) as u32;

        // Player uses debug_menu offsets, others use constants
        let (off_x, off_y) = if entity.kind == EntityKind::Player {
            let (dx, dy) = debug_menu.get_sprite_offset(entity.facing);
            ((dx as f64 * zoom) as i32, (dy as f64 * zoom) as i32)
        } else {
            let ox = (config::ENTITY_OFFSET_X as f64 * zoom) as i32;
            let oy = (config::ENTITY_OFFSET_Y as f64 * zoom) as i32;
            (ox, oy)
        };

        let dst = Rect::new(
            cx - w as i32 / 2 + off_x,
            cy + th / 2 - h as i32 + off_y,
            w,
            h,
        );

        // If this entity is in front of the player and overlaps, draw semi-transparent
        if entity.kind != EntityKind::Player {
            if let Some(pr) = player_rect {
                if dst.has_intersection(pr) {
                    texture.set_alpha_mod(128);
                } else {
                    texture.set_alpha_mod(255);
                }
            }
        }

        let _ = canvas.copy(texture, src_rect, dst);
        texture.set_alpha_mod(255); // reset for next use

        // Return player rect for occlusion checks
        let result = if entity.kind == EntityKind::Player { Some(dst) } else { None };

        // Interaction highlight for non-player entities
        if entity.kind != EntityKind::Player {
            let is_adjacent = (entity.grid_x - highlight.player_x).abs() + (entity.grid_y - highlight.player_y).abs() == 1;
            let is_hovered = highlight.hover_tile == Some((entity.grid_x, entity.grid_y));

            if is_adjacent || is_hovered {
                let color = entity_highlight_color(entity.kind);
                let alpha = if is_adjacent { config::HIGHLIGHT_ALPHA_ADJACENT } else { config::HIGHLIGHT_ALPHA_HOVER };
                let px = config::HIGHLIGHT_OUTLINE_PX;
                let half = px / 2;

                // Re-fetch texture and draw tinted overlay copies (shifted in 8+ directions = outline)
                if let Some(tex2) = assets.get_mut(&key) {
                    tex2.set_color_mod(color.r, color.g, color.b);
                    tex2.set_alpha_mod(alpha);

                    // Draw the sprite offset in cardinal + diagonal directions at full and half thickness
                    for &(ox, oy) in &[(-px,0),(px,0),(0,-px),(0,px),(-px,-px),(px,-px),(-px,px),(px,px),(-half,0),(half,0),(0,-half),(0,half)] {
                        let outline_dst = Rect::new(dst.x() + ox, dst.y() + oy, dst.width(), dst.height());
                        let _ = canvas.copy(tex2, src_rect, outline_dst);
                    }

                    // Reset and redraw the original sprite on top
                    tex2.set_color_mod(255, 255, 255);
                    tex2.set_alpha_mod(255);
                    let b2 = (brightness * 255.0) as u8;
                    tex2.set_color_mod(b2, b2, b2);
                    let _ = canvas.copy(tex2, src_rect, dst);
                    tex2.set_color_mod(255, 255, 255);
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
                format!("[TAB] Mode: per-direction ({:03})", entity.facing)
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

    let (key, src_rect) = entity_texture_info(player);
    let texture = assets.get_mut(&key)?;
    let entity_zoom = zoom * config::ENTITY_SCALE;
    let th = (TILE_HEIGHT as f64 * zoom) as i32;

    let (src_w, src_h) = match src_rect {
        Some(r) => (r.width(), r.height()),
        None => {
            let q = texture.query();
            (q.width, q.height)
        }
    };
    let w = (src_w as f64 * entity_zoom) as u32;
    let h = (src_h as f64 * entity_zoom) as u32;

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
            if let Some(pr) = draw_entity(canvas, assets, &state.entities[entity_idx], &state.fov_map, cam, sw, sh, zoom, debug_menu, text, player_rect, &highlight) {
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
        if let Some(pr) = draw_entity(canvas, assets, &state.entities[entity_idx], &state.fov_map, cam, sw, sh, zoom, debug_menu, text, player_rect, &highlight) {
            player_rect = Some(pr);
        }
        entity_cursor += 1;
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

    // Draw dialogue box if active
    if let Some(dialogue) = &state.active_dialogue {
        draw_dialogue_box(canvas, text, &dialogue.target_name, &dialogue.text, sw, sh);
    }
}

/// Draw a dialogue box at the bottom of the screen.
/// Shows the speaker name and their text in a semi-transparent box.
fn draw_dialogue_box(
    canvas: &mut Canvas<Window>,
    text: &mut TextRenderer,
    speaker: &str,
    dialogue_text: &str,
    sw: i32,
    sh: i32,
) {
    let box_height = config::DIALOGUE_BOX_HEIGHT;
    let margin = config::DIALOGUE_BOX_MARGIN;
    let padding = config::DIALOGUE_BOX_PADDING;

    // Semi-transparent dark background
    canvas.set_draw_color(config::DIALOGUE_BG_COLOR);
    let box_rect = Rect::new(margin, sh - box_height - margin, (sw - margin * 2) as u32, box_height as u32);
    let _ = canvas.fill_rect(box_rect);

    // Border
    canvas.set_draw_color(config::DIALOGUE_BORDER_COLOR);
    let _ = canvas.draw_rect(box_rect);

    // Speaker name (yellow, larger font)
    let name_x = margin + padding;
    let name_y = sh - box_height - margin + padding;
    if let Some(name_tex) = text.render(speaker, config::DIALOGUE_NAME_FONT_SIZE, config::DIALOGUE_NAME_COLOR) {
        let q = name_tex.query();
        let dst = Rect::new(name_x, name_y, q.width, q.height);
        let _ = canvas.copy(name_tex, None, dst);
    }

    // Dialogue text (white, smaller font)
    let text_x = margin + padding;
    let text_y = name_y + config::DIALOGUE_TEXT_GAP;
    if let Some(text_tex) = text.render(dialogue_text, config::DIALOGUE_TEXT_FONT_SIZE, config::DIALOGUE_TEXT_COLOR) {
        let q = text_tex.query();
        let dst = Rect::new(text_x, text_y, q.width, q.height);
        let _ = canvas.copy(text_tex, None, dst);
    }

    // Hint text
    let hint = "[E] Cerrar";
    let hint_x = margin + padding;
    let hint_y = sh - margin - padding - config::DIALOGUE_HINT_BOTTOM_GAP;
    if let Some(hint_tex) = text.render(hint, config::DIALOGUE_HINT_FONT_SIZE, config::DIALOGUE_HINT_COLOR) {
        let q = hint_tex.query();
        let dst = Rect::new(hint_x, hint_y, q.width, q.height);
        let _ = canvas.copy(hint_tex, None, dst);
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
