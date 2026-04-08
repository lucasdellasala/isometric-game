use std::collections::HashMap;

/// Sprite offset debug tool. Toggle with F2.
/// Allows adjusting X/Y offsets per facing direction at runtime.
/// When done, prints the final values to console so you can paste them into code.
pub struct SpriteDebug {
    pub active: bool,
    /// Per-direction offsets: key = facing angle (0, 45, ..., 315), value = (offset_x, offset_y)
    pub offsets: HashMap<u16, (i32, i32)>,
    /// Base offset applied to all directions
    pub base_offset_x: i32,
    pub base_offset_y: i32,
    /// Whether we're editing the base offset (false) or per-direction offset (true)
    pub per_direction_mode: bool,
}

impl SpriteDebug {
    pub fn new(base_x: i32, base_y: i32) -> SpriteDebug {
        let mut offsets = HashMap::new();
        for angle in (0..360).step_by(45) {
            offsets.insert(angle as u16, (0, 0));
        }
        SpriteDebug {
            active: false,
            offsets,
            base_offset_x: base_x,
            base_offset_y: base_y,
            per_direction_mode: false,
        }
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
        if !self.active {
            self.print_offsets();
        }
    }

    /// Toggle between base offset mode and per-direction mode
    pub fn toggle_mode(&mut self) {
        self.per_direction_mode = !self.per_direction_mode;
    }

    /// Get the total offset for a given facing direction
    pub fn get_offset(&self, facing: u16) -> (i32, i32) {
        let (dx, dy) = self.offsets.get(&facing).copied().unwrap_or((0, 0));
        (self.base_offset_x + dx, self.base_offset_y + dy)
    }

    pub fn adjust(&mut self, facing: u16, dx: i32, dy: i32) {
        if self.per_direction_mode {
            let entry = self.offsets.entry(facing).or_insert((0, 0));
            entry.0 += dx;
            entry.1 += dy;
        } else {
            self.base_offset_x += dx;
            self.base_offset_y += dy;
        }
    }

    /// Print current offsets to console for copy-pasting into code
    fn print_offsets(&self) {
        println!("=== Sprite Debug Offsets ===");
        println!("Base: ({}, {})", self.base_offset_x, self.base_offset_y);
        for angle in (0..360).step_by(45) {
            let (dx, dy) = self.offsets.get(&(angle as u16)).copied().unwrap_or((0, 0));
            if dx != 0 || dy != 0 {
                println!("  {:03}: ({}, {})", angle, dx, dy);
            }
        }
        println!("===========================");
    }
}
