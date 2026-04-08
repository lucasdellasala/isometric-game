use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::render::assets::AssetManager;

/// A single grass tuft to draw on a tile.
pub struct GrassTuft {
    pub sprite_index: usize, // 0..7 → grass_tuft_01 to _08
    pub offset_x: i32,       // pixels relative to tile center
    pub offset_y: i32,       // pixels relative to tile center (negative = back, positive = front)
}

/// Simple inline LCG pseudo-random number generator.
/// Like a seeded Math.random() in JS — deterministic, same seed = same sequence.
struct Lcg {
    state: u32,
}

impl Lcg {
    fn new(seed: u32) -> Lcg {
        Lcg { state: seed.wrapping_add(1) }
    }

    /// Returns a pseudo-random u32.
    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state >> 16) & 0x7fff
    }

    /// Returns a value in [min, max] inclusive.
    fn range(&mut self, min: i32, max: i32) -> i32 {
        let span = (max - min + 1) as u32;
        (self.next() % span) as i32 + min
    }
}

/// Generate grass tufts for a tile at (col, row).
/// Returns 0-3 tufts with deterministic random positions.
/// Only call this for TileKind::Grass tiles.
pub fn generate_grass_tufts(col: i32, row: i32) -> Vec<GrassTuft> {
    let seed = (col as u32).wrapping_mul(7919).wrapping_add((row as u32).wrapping_mul(6271));
    let mut rng = Lcg::new(seed);

    // Weighted count: 20% chance 0, 40% chance 1, 30% chance 2, 10% chance 3
    let roll = rng.next() % 100;
    let count = if roll < 20 { 0 } else if roll < 60 { 1 } else if roll < 90 { 2 } else { 3 };

    let mut tufts = Vec::with_capacity(count);
    for _ in 0..count {
        tufts.push(GrassTuft {
            sprite_index: (rng.next() % 8) as usize,
            offset_x: rng.range(-16, 16),
            offset_y: rng.range(-8, 8),
        });
    }
    tufts
}

/// Draw grass tufts at a tile's screen position.
/// tile_cx, tile_cy = the screen position returned by to_screen() for this tile.
/// brightness = FOV brightness for darkening.
/// zoom = current camera zoom.
/// filter: if Some(true), only draw tufts with offset_y < 0 (behind entities).
///         if Some(false), only draw tufts with offset_y >= 0 (in front of entities).
///         if None, draw all.
pub fn draw_grass_tufts(
    canvas: &mut Canvas<Window>,
    assets: &mut AssetManager,
    tufts: &[GrassTuft],
    tile_cx: i32,
    tile_cy: i32,
    brightness: f64,
    zoom: f64,
    filter: Option<bool>,
    player_rect: Option<Rect>,
) {
    let b = (brightness * 255.0) as u8;

    for tuft in tufts {
        // Filter: behind (offset_y < 0) or front (offset_y >= 0)
        match filter {
            Some(true) if tuft.offset_y >= 0 => continue,  // want behind, skip front
            Some(false) if tuft.offset_y < 0 => continue,  // want front, skip behind
            _ => {}
        }

        let key = format!("grass_tuft_{:02}", tuft.sprite_index + 1);
        if let Some(texture) = assets.get_mut(&key) {
            let query = texture.query();
            let w = (query.width as f64 * zoom) as u32;
            let h = (query.height as f64 * zoom) as u32;
            let ox = (tuft.offset_x as f64 * zoom) as i32;
            let oy = (tuft.offset_y as f64 * zoom) as i32;

            // Anchor at base of sprite (bottom-center), offset from tile center
            let dst = Rect::new(
                tile_cx + ox - w as i32 / 2,
                tile_cy + oy + (32.0 * zoom) as i32 - h as i32, // 32 = TILE_HEIGHT/2, center of diamond
                w,
                h,
            );

            // Semi-transparent if overlapping with the player
            if let Some(pr) = player_rect {
                if dst.has_intersection(pr) {
                    texture.set_alpha_mod(128);
                } else {
                    texture.set_alpha_mod(255);
                }
            }

            texture.set_color_mod(b, b, b);
            let _ = canvas.copy(texture, None, dst);
            texture.set_alpha_mod(255); // reset
        }
    }
}
