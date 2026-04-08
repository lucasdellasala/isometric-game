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
}

impl<'a> AssetManager<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> AssetManager<'a> {
        AssetManager {
            texture_creator,
            textures: HashMap::new(),
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

    /// Get a mutable texture by key (needed to set color mod for FOV darkening).
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Texture<'a>> {
        self.textures.get_mut(key)
    }

    /// Load real assets where available, generate placeholders for the rest.
    pub fn generate_placeholders(&mut self) -> Result<(), String> {
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

        // --- Player sprites ---
        // Idle: 8 directional sprites
        for angle in (0..360).step_by(45) {
            let key = format!("entity_player_{angle:03}");
            let path = format!("assets/sprites/player/idle/entity_player_{angle:03}.png");
            if self.load_image(&key, &path).is_err() {
                self.create_entity_texture(&key, Color::RGB(200, 60, 60))?;
            }
        }

        // Walk: 8 directions × 8 frames
        for angle in (0..360).step_by(45) {
            for frame in 0..8 {
                let key = format!("entity_player_walk_{angle:03}_{frame}");
                let path = format!("assets/sprites/player/walk/entity_player_walk_{angle:03}_{frame}.png");
                let _ = self.load_image(&key, &path);
            }
        }

        // --- NPC spritesheets (9 variants, 1024x256 each, 8 directional frames) ---
        let npc_variants = [
            "african_black", "african_brown", "african_cream",
            "caucasian_black", "caucasian_brown", "caucasian_cream",
            "latino_black", "latino_brown", "latino_cream",
        ];
        for variant in &npc_variants {
            let key = format!("npc_{variant}");
            let path = format!("assets/sprites/npc/entity_npc_{variant}.png");
            if self.load_image(&key, &path).is_err() {
                self.create_entity_texture(&key, Color::RGB(60, 60, 200))?;
            }
        }
        // Legacy single NPC sprite as fallback
        if self.load_image("entity_npc", "assets/sprites/npc/entity_npc.png").is_err() {
            self.create_entity_texture("entity_npc", Color::RGB(60, 60, 200))?;
        }

        // --- Enemy sprites ---
        if self.load_image("entity_enemy", "assets/sprites/enemy/entity_enemy.png").is_err() {
            self.create_entity_texture("entity_enemy", Color::RGB(200, 60, 200))?;
        }

        // --- Decoration sprites ---
        for i in 1..=8 {
            let _ = self.load_image(
                &format!("grass_tuft_{i:02}"),
                &format!("assets/sprites/decorations/grass_tuft_{i:02}.png"),
            );
        }

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
