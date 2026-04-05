/// Camera controls the viewport offset.
/// Everything drawn on screen is shifted by (x, y).
pub struct Camera {
    pub x: i32,
    pub y: i32,
    pub speed: i32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            x: 0,
            y: 0,
            speed: 5,
        }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx * self.speed;
        self.y += dy * self.speed;
    }
}
