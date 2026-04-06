use rusttype::{Font, Scale, point};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use std::collections::HashMap;

/// Renders text strings into SDL2 textures using rusttype (pure Rust, no SDL2_ttf).
///
/// Think of it like a mini canvas.measureText() + canvas.fillText() from JS:
/// rusttype rasterizes the font glyphs into a pixel buffer, and we upload that
/// buffer to an SDL2 texture.
pub struct TextRenderer<'a> {
    font: Font<'static>,
    texture_creator: &'a TextureCreator<WindowContext>,
    cache: HashMap<TextCacheKey, Texture<'a>>,
}

/// Cache key: we cache by (text, size, color) so we don't re-rasterize every frame.
#[derive(Hash, Eq, PartialEq)]
struct TextCacheKey {
    text: String,
    size: u32,
    color_r: u8,
    color_g: u8,
    color_b: u8,
}

impl<'a> TextRenderer<'a> {
    /// Create a new TextRenderer. Loads a .ttf font from disk.
    /// Falls back to a bundled font if the file is not found.
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        font_path: &str,
    ) -> Result<TextRenderer<'a>, String> {
        let font_data = std::fs::read(font_path)
            .map_err(|e| format!("Failed to read font {font_path}: {e}"))?;

        // Font::try_from_vec takes ownership of the Vec<u8>.
        // The 'static lifetime works because the font data is owned by the Font struct.
        let font = Font::try_from_vec(font_data)
            .ok_or_else(|| format!("Failed to parse font {font_path}"))?;

        Ok(TextRenderer {
            font,
            texture_creator,
            cache: HashMap::new(),
        })
    }

    /// Render a text string and return a reference to the cached texture.
    /// Returns None if the text is empty.
    pub fn render(
        &mut self,
        text: &str,
        size: u32,
        color: Color,
    ) -> Option<&Texture<'a>> {
        if text.is_empty() {
            return None;
        }

        let key = TextCacheKey {
            text: String::from(text),
            size,
            color_r: color.r,
            color_g: color.g,
            color_b: color.b,
        };

        // Return cached texture if available
        if self.cache.contains_key(&key) {
            return self.cache.get(&key);
        }

        // Rasterize the text
        let scale = Scale::uniform(size as f32);
        let v_metrics = self.font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);

        let glyphs: Vec<_> = self.font.layout(text, scale, offset).collect();

        // Calculate bounding box
        let width = glyphs
            .iter()
            .rev()
            .filter_map(|g| {
                g.pixel_bounding_box().map(|bb| bb.max.x)
            })
            .next()
            .unwrap_or(0) as u32;

        let height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;

        if width == 0 || height == 0 {
            return None;
        }

        // Create pixel buffer (RGBA)
        let mut pixels = vec![0u8; (width * height * 4) as usize];

        for glyph in &glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = (gx as i32 + bb.min.x) as u32;
                    let py = (gy as i32 + bb.min.y) as u32;
                    if px < width && py < height {
                        let alpha = (v * 255.0) as u8;
                        let offset = ((py * width + px) * 4) as usize;
                        // ABGR8888 byte order (matches SDL2 surface from image crate)
                        pixels[offset] = color.r;
                        pixels[offset + 1] = color.g;
                        pixels[offset + 2] = color.b;
                        pixels[offset + 3] = alpha;
                    }
                });
            }
        }

        // Create SDL2 surface and texture
        let mut surface = sdl2::surface::Surface::new(width, height, PixelFormatEnum::ABGR8888)
            .ok()?;

        surface.with_lock_mut(|dst: &mut [u8]| {
            dst[..pixels.len()].copy_from_slice(&pixels);
        });

        let texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .ok()?;

        self.cache.insert(key, texture);

        // Return the just-inserted texture
        let key2 = TextCacheKey {
            text: String::from(text),
            size,
            color_r: color.r,
            color_g: color.g,
            color_b: color.b,
        };
        self.cache.get(&key2)
    }

    /// Clear the texture cache (call when text changes frequently to free memory).
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
