use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Post-process mode selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PostProcessMode {
    Off,
    Dithering,
    Moebius,
}

impl PostProcessMode {
    pub fn label(&self) -> &'static str {
        match self {
            PostProcessMode::Off => "Off",
            PostProcessMode::Dithering => "Dithering",
            PostProcessMode::Moebius => "Moebius",
        }
    }

    pub fn next(&self) -> PostProcessMode {
        match self {
            PostProcessMode::Off => PostProcessMode::Dithering,
            PostProcessMode::Dithering => PostProcessMode::Moebius,
            PostProcessMode::Moebius => PostProcessMode::Off,
        }
    }

    pub fn prev(&self) -> PostProcessMode {
        match self {
            PostProcessMode::Off => PostProcessMode::Moebius,
            PostProcessMode::Dithering => PostProcessMode::Off,
            PostProcessMode::Moebius => PostProcessMode::Dithering,
        }
    }
}

/// Apply scope for post-process effects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApplyScope {
    TilesOnly,
    FullScreen,
}

impl ApplyScope {
    pub fn label(&self) -> &'static str {
        match self {
            ApplyScope::TilesOnly => "Tiles only",
            ApplyScope::FullScreen => "Full screen",
        }
    }

    pub fn toggle(&self) -> ApplyScope {
        match self {
            ApplyScope::TilesOnly => ApplyScope::FullScreen,
            ApplyScope::FullScreen => ApplyScope::TilesOnly,
        }
    }
}

/// Bayer 4x4 dithering matrix, normalized to 0.0-1.0.
const BAYER_4X4: [[f64; 4]; 4] = [
    [0.0  / 16.0,  8.0 / 16.0,  2.0 / 16.0, 10.0 / 16.0],
    [12.0 / 16.0,  4.0 / 16.0, 14.0 / 16.0,  6.0 / 16.0],
    [3.0  / 16.0, 11.0 / 16.0,  1.0 / 16.0,  9.0 / 16.0],
    [15.0 / 16.0,  7.0 / 16.0, 13.0 / 16.0,  5.0 / 16.0],
];

/// 16-color palette for quantization.
const PALETTE: [(u8, u8, u8); 16] = [
    (10,  8,   20),
    (35,  25,  45),
    (60,  42,  55),
    (85,  62,  60),
    (110, 82,  65),
    (140, 105, 75),
    (165, 128, 85),
    (190, 152, 100),
    (100, 72,  40),
    (130, 95,  50),
    (160, 120, 60),
    (185, 148, 80),
    (205, 170, 110),
    (220, 190, 145),
    (235, 210, 175),
    (250, 232, 205),
];

/// Adjustable dithering parameters.
pub struct DitherParams {
    /// Multiplier applied to Bayer offset spread. Controls dithering intensity.
    pub brightness_boost: f64,
    /// Tint for light pixels: palette colors are lerped toward this color.
    /// (255,255,255) = no tint, (255,200,150) = warm tint on brights.
    pub color_light: (u8, u8, u8),
    /// Tint for dark pixels: palette colors are lerped toward this color.
    /// (0,0,0) = no tint, (20,10,40) = purple tint on darks.
    pub color_dark: (u8, u8, u8),
}

impl DitherParams {
    pub fn new() -> DitherParams {
        DitherParams {
            brightness_boost: 0.5,
            color_light: (255, 255, 255),
            color_dark: (0, 0, 0),
        }
    }
}

/// Find the closest palette color to (r, g, b) using squared euclidean distance.
fn nearest_palette(r: f64, g: f64, b: f64) -> (u8, u8, u8) {
    let mut best = PALETTE[0];
    let mut best_dist = f64::MAX;

    for &(pr, pg, pb) in &PALETTE {
        let dr = r - pr as f64;
        let dg = g - pg as f64;
        let db = b - pb as f64;
        let dist = dr * dr + dg * dg + db * db;
        if dist < best_dist {
            best_dist = dist;
            best = (pr, pg, pb);
        }
    }

    best
}

/// Apply a tint to a palette color based on its brightness.
/// Dark palette colors are lerped toward color_dark, bright ones toward color_light.
fn apply_tint(pr: u8, pg: u8, pb: u8, params: &DitherParams) -> (u8, u8, u8) {
    // Compute brightness of the palette color (0.0 = darkest, 1.0 = brightest)
    let luma = (0.299 * pr as f64 + 0.587 * pg as f64 + 0.114 * pb as f64) / 255.0;

    let (lr, lg, lb) = params.color_light;
    let (dr, dg, db) = params.color_dark;

    // Lerp: dark tint at luma=0, light tint at luma=1
    // tint_r = dark_r * (1-luma) + light_r * luma
    // Final = palette * 0.6 + tint * 0.4 (40% tint strength)
    let tint_strength = 0.4;
    let tint_r = dr as f64 * (1.0 - luma) + lr as f64 * luma;
    let tint_g = dg as f64 * (1.0 - luma) + lg as f64 * luma;
    let tint_b = db as f64 * (1.0 - luma) + lb as f64 * luma;

    let out_r = (pr as f64 * (1.0 - tint_strength) + tint_r * tint_strength).clamp(0.0, 255.0) as u8;
    let out_g = (pg as f64 * (1.0 - tint_strength) + tint_g * tint_strength).clamp(0.0, 255.0) as u8;
    let out_b = (pb as f64 * (1.0 - tint_strength) + tint_b * tint_strength).clamp(0.0, 255.0) as u8;

    (out_r, out_g, out_b)
}

/// Apply ordered dithering with 16-color palette quantization.
pub fn apply_dither(canvas: &mut Canvas<Window>, params: &DitherParams) {
    let (w, h) = canvas.output_size().unwrap_or((1280, 900));

    let pitch = w as usize * 3;
    let mut pixels = match canvas.read_pixels(None, PixelFormatEnum::RGB24) {
        Ok(p) => p,
        Err(_) => return,
    };

    let spread = 32.0 * params.brightness_boost;

    for y in 0..h as usize {
        let bayer_row = &BAYER_4X4[y % 4];
        for x in 0..w as usize {
            let offset = y * pitch + x * 3;
            let r = pixels[offset] as f64;
            let g = pixels[offset + 1] as f64;
            let b = pixels[offset + 2] as f64;

            let bayer_offset = (bayer_row[x % 4] - 0.5) * spread;
            let dr = (r + bayer_offset).clamp(0.0, 255.0);
            let dg = (g + bayer_offset).clamp(0.0, 255.0);
            let db = (b + bayer_offset).clamp(0.0, 255.0);

            let (pr, pg, pb) = nearest_palette(dr, dg, db);
            let (fr, fg, fb) = apply_tint(pr, pg, pb, params);
            pixels[offset] = fr;
            pixels[offset + 1] = fg;
            pixels[offset + 2] = fb;
        }
    }

    let texture_creator = canvas.texture_creator();
    let mut texture = match texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24,
        w,
        h,
    ) {
        Ok(t) => t,
        Err(_) => return,
    };

    let _ = texture.with_lock(None, |dst: &mut [u8], _pitch: usize| {
        dst[..pixels.len()].copy_from_slice(&pixels);
    });

    let _ = canvas.copy(&texture, None, None);
}

/// Moebius post-process parameters.
pub struct MoebiusParams {
    /// Number of levels per RGB channel for posterization (2-8).
    pub posterize_levels: u8,
    /// Sobel edge detection threshold. Lower = more edges (5-100).
    pub edge_threshold: u8,
}

impl MoebiusParams {
    pub fn new() -> MoebiusParams {
        MoebiusParams {
            posterize_levels: 4,
            edge_threshold: 30,
        }
    }
}

/// Posterize a single channel value to N levels.
/// With N=4: 0, 85, 170, 255.
fn posterize(value: u8, levels: u8) -> u8 {
    let n = levels as f64;
    let bucket = (value as f64 / (256.0 / n)).floor().min(n - 1.0);
    (bucket * (255.0 / (n - 1.0))).round() as u8
}

/// Compute luma for a pixel at (x, y) in the pixel buffer.
fn pixel_luma(pixels: &[u8], x: usize, y: usize, pitch: usize) -> f64 {
    let offset = y * pitch + x * 3;
    let r = pixels[offset] as f64;
    let g = pixels[offset + 1] as f64;
    let b = pixels[offset + 2] as f64;
    0.299 * r + 0.587 * g + 0.114 * b
}

/// Apply Moebius post-process: posterization + Sobel edge detection.
pub fn apply_moebius(canvas: &mut Canvas<Window>, params: &MoebiusParams) {
    let (w, h) = canvas.output_size().unwrap_or((1280, 900));
    let w = w as usize;
    let h = h as usize;
    let pitch = w * 3;

    let mut pixels = match canvas.read_pixels(None, PixelFormatEnum::RGB24) {
        Ok(p) => p,
        Err(_) => return,
    };

    // Step 1: Posterize each channel
    for i in 0..pixels.len() {
        pixels[i] = posterize(pixels[i], params.posterize_levels);
    }

    // Step 2: Sobel edge detection on the posterized image.
    // We need to read from the posterized pixels and write edges to a separate buffer.
    let mut output = pixels.clone();
    let threshold = params.edge_threshold as f64;

    // Sobel kernels:
    // Gx: [-1 0 1]    Gy: [-1 -2 -1]
    //     [-2 0 2]         [ 0  0  0]
    //     [-1 0 1]         [ 1  2  1]
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let tl = pixel_luma(&pixels, x - 1, y - 1, pitch);
            let tc = pixel_luma(&pixels, x,     y - 1, pitch);
            let tr = pixel_luma(&pixels, x + 1, y - 1, pitch);
            let ml = pixel_luma(&pixels, x - 1, y,     pitch);
            let mr = pixel_luma(&pixels, x + 1, y,     pitch);
            let bl = pixel_luma(&pixels, x - 1, y + 1, pitch);
            let bc = pixel_luma(&pixels, x,     y + 1, pitch);
            let br = pixel_luma(&pixels, x + 1, y + 1, pitch);

            let gx = -tl + tr - 2.0 * ml + 2.0 * mr - bl + br;
            let gy = -tl - 2.0 * tc - tr + bl + 2.0 * bc + br;
            let gradient = (gx * gx + gy * gy).sqrt();

            if gradient > threshold {
                let offset = y * pitch + x * 3;
                output[offset] = 0;
                output[offset + 1] = 0;
                output[offset + 2] = 0;
            }
        }
    }

    // Write back
    let texture_creator = canvas.texture_creator();
    let mut texture = match texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24,
        w as u32,
        h as u32,
    ) {
        Ok(t) => t,
        Err(_) => return,
    };

    let _ = texture.with_lock(None, |dst: &mut [u8], _pitch: usize| {
        dst[..output.len()].copy_from_slice(&output);
    });

    let _ = canvas.copy(&texture, None, None);
}
