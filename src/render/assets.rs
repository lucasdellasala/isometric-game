use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use std::collections::HashMap;
use std::path::Path;

use crate::render::iso::{TILE_HEIGHT, TILE_WIDTH};

/// Manages textures for tiles, entities, and UI.
/// The lifetime 'a is tied to the TextureCreator — textures can't outlive it.
///
/// In JS terms: think of TextureCreator as a database connection,
/// and Texture as a query result. The result is invalid if the connection closes.
pub struct AssetManager<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    textures: HashMap<String, Texture<'a>>,
    /// Spritesheet region mappings: frame key → (sheet texture key, src_rect).
    /// When get_texture() is called, if the key has a region entry, the sheet
    /// texture is returned with the src_rect for that frame.
    sprite_regions: HashMap<String, (String, sdl2::rect::Rect)>,
    /// Pre-computed outline points for sprites.
    outlines: HashMap<String, Vec<(i32, i32)>>,
}

impl<'a> AssetManager<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> AssetManager<'a> {
        AssetManager {
            texture_creator,
            textures: HashMap::new(),
            sprite_regions: HashMap::new(),
            outlines: HashMap::new(),
        }
    }

    /// Load a PNG/JPG image from disk using the `image` crate.
    #[allow(dead_code)]
    pub fn load_image(&mut self, key: &str, path: &str) -> Result<(), String> {
        if !Path::new(path).exists() {
            return Err(format!("File not found: {path}"));
        }

        let img = image::open(path)
            .map_err(|e| format!("Failed to load {path}: {e}"))?
            .to_rgba8();

        let (w, h) = img.dimensions();
        let raw_pixels = img.into_raw();

        let mut surface = sdl2::surface::Surface::new(w, h, PixelFormatEnum::ABGR8888)
            .map_err(|e| format!("Failed to create surface: {e}"))?;

        surface.with_lock_mut(|dst: &mut [u8]| {
            dst[..raw_pixels.len()].copy_from_slice(&raw_pixels);
        });

        let texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| format!("Failed to create texture: {e}"))?;

        self.textures.insert(String::from(key), texture);
        Ok(())
    }

    /// Load an image and generate its outline in one pass (single file read).
    /// Much faster than load_image() + generate_outline_for_image() separately.
    pub fn load_image_with_outline(&mut self, texture_key: &str, outline_key: &str, path: &str) -> Result<(), String> {
        if !Path::new(path).exists() {
            return Err(format!("File not found: {path}"));
        }

        let img = image::open(path)
            .map_err(|e| format!("Failed to load {path}: {e}"))?
            .to_rgba8();

        let (w, h) = img.dimensions();

        // Generate outline from the same pixel data (no second file read)
        let mut points = Vec::new();
        for py in 0..h {
            for px in 0..w {
                let pixel = img.get_pixel(px, py);
                if pixel[3] >= 128 { continue; }
                let is_edge = [(0i32,-1i32),(0,1),(-1,0),(1,0)].iter().any(|&(dx,dy)| {
                    let nx = px as i32 + dx;
                    let ny = py as i32 + dy;
                    if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 { return false; }
                    img.get_pixel(nx as u32, ny as u32)[3] >= 128
                });
                if is_edge { points.push((px as i32, py as i32)); }
            }
        }
        self.outlines.insert(String::from(outline_key), points);

        // Create SDL2 texture
        let raw_pixels = img.into_raw();
        let mut surface = sdl2::surface::Surface::new(w, h, PixelFormatEnum::ABGR8888)
            .map_err(|e| format!("Failed to create surface: {e}"))?;
        surface.with_lock_mut(|dst: &mut [u8]| {
            dst[..raw_pixels.len()].copy_from_slice(&raw_pixels);
        });
        let texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| format!("Failed to create texture: {e}"))?;
        self.textures.insert(String::from(texture_key), texture);

        Ok(())
    }

    /// Get a mutable texture by key. If the key is a spritesheet region,
    /// returns the sheet texture (caller should use get_src_rect to get the region).
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Texture<'a>> {
        if let Some((sheet_key, _)) = self.sprite_regions.get(key) {
            let sk = sheet_key.clone();
            self.textures.get_mut(&sk)
        } else {
            self.textures.get_mut(key)
        }
    }

    /// Get the src_rect for a spritesheet region (None for individual textures).
    pub fn get_src_rect(&self, key: &str) -> Option<sdl2::rect::Rect> {
        self.sprite_regions.get(key).map(|(_, r)| *r)
    }

    /// Check if a texture or spritesheet region is loaded.
    pub fn has_texture(&self, key: &str) -> bool {
        self.textures.contains_key(key) || self.sprite_regions.contains_key(key)
    }

    /// Get pre-computed outline points for a sprite frame.
    /// Key format: "{asset_key}_{frame_index}" (e.g., "npc_african_black_3").
    pub fn get_outline(&self, key: &str) -> Option<&Vec<(i32, i32)>> {
        self.outlines.get(key)
    }

    /// Generate outline points for a spritesheet's frames.
    /// Reads pixel data from the PNG on disk (not from GPU texture),
    /// detects edge pixels (opaque with at least one transparent neighbor),
    /// and stores them for fast rendering later.
    ///
    /// Called once per scene load, not at startup for all assets.
    /// Only generates outlines for assets actually present in the scene.
    pub fn generate_outlines_for_spritesheet(
        &mut self,
        asset_key: &str,
        path: &str,
        frame_w: u32,
        frame_h: u32,
        frame_count: u32,
    ) {
        let img = match image::open(path) {
            Ok(i) => i.to_rgba8(),
            Err(_) => return,
        };

        for frame in 0..frame_count {
            let mut points = Vec::new();
            let fx = frame * frame_w;

            for py in 0..frame_h {
                for px in 0..frame_w {
                    let pixel = img.get_pixel(fx + px, py);
                    if pixel[3] >= 128 {
                        continue; // opaque pixel, skip — we want transparent pixels outside the sprite
                    }

                    // Check if any 4-connected neighbor is opaque → this transparent pixel is just outside the edge
                    let is_outer_edge = [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)].iter().any(|&(dx, dy)| {
                        let nx = (fx + px) as i32 + dx;
                        let ny = py as i32 + dy;
                        if nx < fx as i32 || nx >= (fx + frame_w) as i32 || ny < 0 || ny >= frame_h as i32 {
                            return false;
                        }
                        let neighbor = img.get_pixel(nx as u32, ny as u32);
                        neighbor[3] >= 128
                    });

                    if is_outer_edge {
                        points.push((px as i32, py as i32));
                    }
                }
            }

            let key = format!("{asset_key}_{frame}");
            self.outlines.insert(key, points);
        }
    }

    /// Generate outline points for a single PNG image (not a spritesheet).
    /// Stores with the exact key provided — no "_0" suffix.
    pub fn generate_outline_for_image(&mut self, outline_key: &str, path: &str) {
        let img = match image::open(path) {
            Ok(i) => i.to_rgba8(),
            Err(_) => return,
        };

        let w = img.width();
        let h = img.height();
        let mut points = Vec::new();

        for py in 0..h {
            for px in 0..w {
                let pixel = img.get_pixel(px, py);
                if pixel[3] >= 128 {
                    continue;
                }

                let is_outer_edge = [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)].iter().any(|&(dx, dy)| {
                    let nx = px as i32 + dx;
                    let ny = py as i32 + dy;
                    if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 {
                        return false;
                    }
                    let neighbor = img.get_pixel(nx as u32, ny as u32);
                    neighbor[3] >= 128
                });

                if is_outer_edge {
                    points.push((px as i32, py as i32));
                }
            }
        }

        self.outlines.insert(String::from(outline_key), points);
    }

    /// Load a spritesheet PNG and register individual frame keys.
    /// Layout: 8 columns (directions S,SW,W,NW,N,NE,E,SE) × num_frames rows.
    /// Each frame is frame_w × frame_h pixels.
    /// Registers keys as "{key_prefix}_{DIR}_{frame}" for multi-frame,
    /// or "{key_prefix}_{DIR}" for single-frame sheets.
    fn load_spritesheet_frames(
        &mut self,
        sheet_key: &str,
        path: &str,
        key_prefix: &str,
        frame_w: u32,
        frame_h: u32,
        num_frames: u32,
    ) -> Result<(), String> {
        self.load_image(sheet_key, path)?;

        let directions = ["S", "SW", "W", "NW", "N", "NE", "E", "SE"];
        for frame in 0..num_frames {
            for (dir_idx, dir) in directions.iter().enumerate() {
                let frame_key = if num_frames == 1 {
                    format!("{key_prefix}_{dir}")
                } else {
                    format!("{key_prefix}_{dir}_{frame}")
                };
                let src = sdl2::rect::Rect::new(
                    (dir_idx as u32 * frame_w) as i32,
                    (frame * frame_h) as i32,
                    frame_w,
                    frame_h,
                );
                self.sprite_regions.insert(frame_key, (String::from(sheet_key), src));
            }
        }
        Ok(())
    }

    /// Load real assets where available, generate placeholders for the rest.
    /// Writes detailed timing to boot_assets.log.
    pub fn generate_placeholders(&mut self) -> Result<(), String> {
        let mut asset_log = Vec::<(String, std::time::Duration)>::new();
        let t = std::time::Instant::now();
        // --- Ground tiles (pre-extracted 128x64 PNGs) ---
        // Grass: 3 variants from forest spritesheet
        for i in 1..=3 {
            let idx = i + 3; // tiles 04, 05, 06
            if self.load_image(&format!("tile_grass_{i}"), &format!("assets/tiles/forest/forest_{idx:02}.png")).is_err() {
                self.create_tile_texture(&format!("tile_grass_{i}"), Color::RGB(80, 150, 80), Color::RGB(60, 120, 60))?;
            }
        }
        // Water: 18 variants
        for i in 1..=18 {
            if self.load_image(&format!("tile_water_{i}"), &format!("assets/tiles/water/water_{i:02}.png")).is_err() {
                self.create_tile_texture(&format!("tile_water_{i}"), Color::RGB(60, 100, 180), Color::RGB(40, 75, 150))?;
            }
        }
        // Dirt: 3 variants from terrain spritesheet
        for i in 1..=3 {
            let idx = i + 3; // tiles 04, 05, 06
            if self.load_image(&format!("tile_dirt_{i}"), &format!("assets/tiles/terrain/terrain_{idx:02}.png")).is_err() {
                self.create_tile_texture(&format!("tile_dirt_{i}"), Color::RGB(150, 120, 70), Color::RGB(120, 95, 50))?;
            }
        }
        // Stone: same as dirt variants
        for i in 1..=3 {
            let idx = i + 3;
            if self.load_image(&format!("tile_stone_{i}"), &format!("assets/tiles/terrain/terrain_{idx:02}.png")).is_err() {
                self.create_tile_texture(&format!("tile_stone_{i}"), Color::RGB(160, 160, 160), Color::RGB(140, 140, 140))?;
            }
        }

        // Wall placeholder (generated, no sprite yet)
        self.create_tile_texture("tile_wall_top", Color::RGB(160, 160, 160), Color::RGB(140, 140, 140))?;

        asset_log.push(("Ground tiles".into(), t.elapsed()));
        let t = std::time::Instant::now();

        // --- Player sprites (from spritesheets) ---
        let sheets = "assets/spritesheets";
        let _ = self.load_spritesheet_frames("_sheet_player_idle", &format!("{sheets}/player_idle.png"), "entity_player", 256, 512, 1);
        let _ = self.load_spritesheet_frames("_sheet_player_idle_anim", &format!("{sheets}/player_idle_anim.png"), "entity_player_idle", 256, 512, 8);
        let _ = self.load_spritesheet_frames("_sheet_player_walk", &format!("{sheets}/player_walk.png"), "entity_player_walk", 256, 512, 8);

        // Generate outlines from the static idle spritesheet (read pixels once)
        let directions = ["S", "SW", "W", "NW", "N", "NE", "E", "SE"];
        for (i, dir) in directions.iter().enumerate() {
            let path = format!("assets/sprites/player/idle/entity_player_{dir}.png");
            let outline_key = format!("entity_player_{i}");
            self.generate_outline_for_image(&outline_key, &path);
        }

        asset_log.push(("Player sprites".into(), t.elapsed()));
        let t = std::time::Instant::now();

        // --- NPC sprites (from spritesheets) ---
        let npc_variants = [
            "african_cr_bk", "african_gn_cr",
            "caucasian_gn_bn", "caucasian_yl_bk",
            "latino_bk_bn", "latino_yl_bk",
        ];
        for variant in &npc_variants {
            let _ = self.load_spritesheet_frames(
                &format!("_sheet_npc_{variant}_idle"), &format!("{sheets}/npc_{variant}_idle.png"),
                &format!("npc_{variant}"), 256, 512, 1);
            let _ = self.load_spritesheet_frames(
                &format!("_sheet_npc_{variant}_idle_anim"), &format!("{sheets}/npc_{variant}_idle_anim.png"),
                &format!("npc_{variant}_idle"), 256, 512, 8);
            let _ = self.load_spritesheet_frames(
                &format!("_sheet_npc_{variant}_walk"), &format!("{sheets}/npc_{variant}_walk.png"),
                &format!("npc_{variant}_walk"), 256, 512, 8);

            // Outlines from static idle (individual PNGs, read once each)
            for (i, dir) in directions.iter().enumerate() {
                let path = format!("assets/sprites/npc/{variant}/entity_npc_{variant}_{dir}.png");
                let outline_key = format!("npc_{variant}_{i}");
                self.generate_outline_for_image(&outline_key, &path);
            }
        }
        self.create_entity_texture("entity_npc", Color::RGB(60, 60, 200))?;

        asset_log.push(("NPC sprites".into(), t.elapsed()));
        let t = std::time::Instant::now();

        // --- Enemy sprites (from spritesheets) ---
        let enemy_types = [("orc", "entity_orc")];
        for (enemy_type, _file_prefix) in &enemy_types {
            let _ = self.load_spritesheet_frames(
                &format!("_sheet_enemy_{enemy_type}_idle"), &format!("{sheets}/enemy_{enemy_type}_idle.png"),
                &format!("enemy_{enemy_type}"), 256, 512, 1);
            let _ = self.load_spritesheet_frames(
                &format!("_sheet_enemy_{enemy_type}_idle_anim"), &format!("{sheets}/enemy_{enemy_type}_idle_anim.png"),
                &format!("enemy_{enemy_type}_idle"), 256, 512, 8);
            let _ = self.load_spritesheet_frames(
                &format!("_sheet_enemy_{enemy_type}_walk"), &format!("{sheets}/enemy_{enemy_type}_walk.png"),
                &format!("enemy_{enemy_type}_walk"), 256, 512, 8);

            for (i, dir) in directions.iter().enumerate() {
                let path = format!("assets/sprites/enemy/{enemy_type}/entity_{enemy_type}_{dir}.png");
                let outline_key = format!("enemy_{enemy_type}_{i}");
                self.generate_outline_for_image(&outline_key, &path);
            }
        }

        asset_log.push(("Enemy sprites".into(), t.elapsed()));
        let t = std::time::Instant::now();
        // --- Entity shadow ---
        // Shared shadow sprite rendered beneath all entities.
        // Will be 256×128 when re-rendered; currently 128×64.
        let _ = self.load_image("entity_shadow", "assets/sprites/player/entity_shadow.png");

        asset_log.push(("Shadow + misc".into(), t.elapsed()));
        let t = std::time::Instant::now();
        // --- Decoration sprites ---
        for i in 1..=8 {
            let _ = self.load_image(
                &format!("grass_tuft_{i:02}"),
                &format!("assets/sprites/decorations/grass_tuft_{i:02}.png"),
            );
        }

        asset_log.push(("Decorations".into(), t.elapsed()));

        // Append asset loading timing log
        let mut log_text = String::from("\nAsset loading breakdown:\n");
        for (label, dur) in &asset_log {
            log_text += &format!("  {:<30} {:.3}s\n", label, dur.as_secs_f64());
        }
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("boot_assets.log") {
            let _ = f.write_all(log_text.as_bytes());
        }
        println!("{log_text}");

        Ok(())
    }

    /// Create an isometric diamond tile texture with two-tone coloring.
    fn create_tile_texture(&mut self, key: &str, color_top: Color, color_bottom: Color) -> Result<(), String> {
        let w = TILE_WIDTH as u32;
        let h = TILE_HEIGHT as u32;

        let mut surface = sdl2::surface::Surface::new(w, h, PixelFormatEnum::RGBA8888)
            .map_err(|e| format!("Failed to create surface: {e}"))?;

        // Fill with transparent
        surface.fill_rect(None, Color::RGBA(0, 0, 0, 0))
            .map_err(|e| format!("Failed to clear surface: {e}"))?;

        // Draw diamond pixel by pixel on the surface
        let half_w = w as i32 / 2;
        let half_h = h as i32 / 2;

        surface.with_lock_mut(|pixels: &mut [u8]| {
            let pitch = w as usize * 4; // RGBA = 4 bytes per pixel

            for py in 0..h as i32 {
                for px in 0..w as i32 {
                    // Check if this pixel is inside the diamond
                    let dx = (px - half_w).abs() as f64 / half_w as f64;
                    let dy = (py - half_h).abs() as f64 / half_h as f64;

                    if dx + dy <= 1.0 {
                        // Inside diamond — pick color based on vertical position
                        let color = if py < half_h { color_top } else { color_bottom };

                        let offset = (py as usize * pitch) + (px as usize * 4);
                        // RGBA8888 byte order: R, G, B, A (on little-endian: A, B, G, R in memory)
                        // SDL surface byte order depends on endianness, use direct RGBA
                        pixels[offset] = 0xFF; // A (in RGBA8888 the first byte is actually A on some platforms)
                        pixels[offset + 1] = color.b;
                        pixels[offset + 2] = color.g;
                        pixels[offset + 3] = color.r;
                    }
                }
            }
        });

        let texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| format!("Failed to create texture from surface: {e}"))?;

        self.textures.insert(String::from(key), texture);
        Ok(())
    }

    /// Create a small solid-color texture (for wall side faces).
    fn create_solid_texture(&mut self, key: &str, color: Color) -> Result<(), String> {
        let w: u32 = 4;
        let h: u32 = 4;

        let mut surface = sdl2::surface::Surface::new(w, h, PixelFormatEnum::RGBA8888)
            .map_err(|e| format!("Failed to create surface: {e}"))?;

        surface.with_lock_mut(|pixels: &mut [u8]| {
            let pitch = w as usize * 4;
            for py in 0..h as usize {
                for px in 0..w as usize {
                    let offset = py * pitch + px * 4;
                    pixels[offset] = 0xFF;
                    pixels[offset + 1] = color.b;
                    pixels[offset + 2] = color.g;
                    pixels[offset + 3] = color.r;
                }
            }
        });

        let texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| format!("Failed to create texture from surface: {e}"))?;

        self.textures.insert(String::from(key), texture);
        Ok(())
    }

    /// Create a simple entity sprite (body + head shape).
    fn create_entity_texture(&mut self, key: &str, body_color: Color) -> Result<(), String> {
        let w: u32 = 32;
        let h: u32 = 40;

        let mut surface = sdl2::surface::Surface::new(w, h, PixelFormatEnum::RGBA8888)
            .map_err(|e| format!("Failed to create surface: {e}"))?;

        surface.fill_rect(None, Color::RGBA(0, 0, 0, 0))
            .map_err(|e| format!("Failed to clear surface: {e}"))?;

        let head_color = Color::RGB(240, 200, 150);

        surface.with_lock_mut(|pixels: &mut [u8]| {
            let pitch = w as usize * 4;

            // Draw head (circle at top center)
            let head_cx = w as i32 / 2;
            let head_cy = 8;
            let head_r = 6;

            for py in 0..h as i32 {
                for px in 0..w as i32 {
                    let offset = (py as usize * pitch) + (px as usize * 4);

                    // Head: circle
                    let dx = px - head_cx;
                    let dy = py - head_cy;
                    if dx * dx + dy * dy <= head_r * head_r {
                        pixels[offset] = 0xFF;
                        pixels[offset + 1] = head_color.b;
                        pixels[offset + 2] = head_color.g;
                        pixels[offset + 3] = head_color.r;
                        continue;
                    }

                    // Body: rectangle below head
                    let body_left = w as i32 / 2 - 6;
                    let body_right = w as i32 / 2 + 6;
                    let body_top = 14;
                    let body_bottom = 36;

                    if px >= body_left && px <= body_right && py >= body_top && py <= body_bottom {
                        pixels[offset] = 0xFF;
                        pixels[offset + 1] = body_color.b;
                        pixels[offset + 2] = body_color.g;
                        pixels[offset + 3] = body_color.r;
                    }
                }
            }
        });

        let texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| format!("Failed to create texture from surface: {e}"))?;

        self.textures.insert(String::from(key), texture);
        Ok(())
    }
}
