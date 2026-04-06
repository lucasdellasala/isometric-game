/// How close the camera is. 1.0 = default, 2.0 = twice as close.
/// Increase this to see less map but bigger tiles.
pub const CAMERA_ZOOM: f64 = 2.0;

/// Camera controls the viewport offset.
/// Everything drawn on screen is shifted by (x, y) and scaled by CAMERA_ZOOM.
pub struct Camera {
    pub x: i32,
    pub y: i32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera { x: 0, y: 0 }
    }
}
