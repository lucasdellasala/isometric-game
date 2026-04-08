/// Camera controls the viewport offset.
/// Everything drawn on screen is shifted by (x, y) and scaled by zoom.
pub struct Camera {
    pub x: i32,
    pub y: i32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera { x: 0, y: 0 }
    }
}
