use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::render::post_process::{ApplyScope, PostProcessMode};
use crate::render::text::TextRenderer;

/// All possible menu option identifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuOption {
    Mode,
    Scope,
    // Dithering options
    Spread,
    LightR,
    LightG,
    LightB,
    DarkR,
    DarkG,
    DarkB,
    // Moebius options
    PosterizeLevels,
    EdgeThreshold,
}

/// Configuration menu state. Toggled with F1, navigated with arrow keys.
pub struct ConfigMenu {
    pub visible: bool,
    selected: usize,
    // Post-process mode
    pub mode: PostProcessMode,
    pub scope: ApplyScope,
    // Dither parameters
    pub spread: f64,
    pub light_r: u8,
    pub light_g: u8,
    pub light_b: u8,
    pub dark_r: u8,
    pub dark_g: u8,
    pub dark_b: u8,
    // Moebius parameters
    pub posterize_levels: u8,
    pub edge_threshold: u8,
}

impl ConfigMenu {
    pub fn new() -> ConfigMenu {
        ConfigMenu {
            visible: false,
            selected: 0,
            mode: PostProcessMode::Off,
            scope: ApplyScope::TilesOnly,
            spread: 0.5,
            light_r: 250,
            light_g: 232,
            light_b: 205,
            dark_r: 35,
            dark_g: 25,
            dark_b: 45,
            posterize_levels: 4,
            edge_threshold: 30,
        }
    }

    /// Get the list of currently visible menu options based on mode.
    fn visible_options(&self) -> Vec<MenuOption> {
        let mut opts = vec![MenuOption::Mode];

        match self.mode {
            PostProcessMode::Off => {}
            PostProcessMode::Dithering => {
                opts.push(MenuOption::Scope);
                opts.push(MenuOption::Spread);
                opts.push(MenuOption::LightR);
                opts.push(MenuOption::LightG);
                opts.push(MenuOption::LightB);
                opts.push(MenuOption::DarkR);
                opts.push(MenuOption::DarkG);
                opts.push(MenuOption::DarkB);
            }
            PostProcessMode::Moebius => {
                opts.push(MenuOption::Scope);
                opts.push(MenuOption::PosterizeLevels);
                opts.push(MenuOption::EdgeThreshold);
            }
        }

        opts
    }

    /// Get the label for a menu option.
    fn option_label(&self, opt: MenuOption) -> String {
        match opt {
            MenuOption::Mode => format!("Mode:             {}", self.mode.label()),
            MenuOption::Scope => format!("Apply to:         {}", self.scope.label()),
            MenuOption::Spread => format!("Spread:           {:.1}", self.spread),
            MenuOption::LightR => format!("Light R:          {}", self.light_r),
            MenuOption::LightG => format!("Light G:          {}", self.light_g),
            MenuOption::LightB => format!("Light B:          {}", self.light_b),
            MenuOption::DarkR => format!("Dark R:           {}", self.dark_r),
            MenuOption::DarkG => format!("Dark G:           {}", self.dark_g),
            MenuOption::DarkB => format!("Dark B:           {}", self.dark_b),
            MenuOption::PosterizeLevels => format!("Posterize levels: {}", self.posterize_levels),
            MenuOption::EdgeThreshold => format!("Edge threshold:   {}", self.edge_threshold),
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn move_up(&mut self) {
        let count = self.visible_options().len();
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = count - 1;
        }
    }

    pub fn move_down(&mut self) {
        let count = self.visible_options().len();
        if self.selected < count - 1 {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
    }

    pub fn adjust_right(&mut self) {
        let opts = self.visible_options();
        let opt = opts[self.selected];
        match opt {
            MenuOption::Mode => {
                self.mode = self.mode.next();
                // Clamp selected index when options change
                let new_count = self.visible_options().len();
                if self.selected >= new_count {
                    self.selected = new_count - 1;
                }
            }
            MenuOption::Scope => self.scope = self.scope.toggle(),
            MenuOption::Spread => self.spread = (self.spread + 0.1).min(3.0),
            MenuOption::LightR => self.light_r = self.light_r.saturating_add(5),
            MenuOption::LightG => self.light_g = self.light_g.saturating_add(5),
            MenuOption::LightB => self.light_b = self.light_b.saturating_add(5),
            MenuOption::DarkR => self.dark_r = self.dark_r.saturating_add(5),
            MenuOption::DarkG => self.dark_g = self.dark_g.saturating_add(5),
            MenuOption::DarkB => self.dark_b = self.dark_b.saturating_add(5),
            MenuOption::PosterizeLevels => self.posterize_levels = (self.posterize_levels + 1).min(8),
            MenuOption::EdgeThreshold => self.edge_threshold = self.edge_threshold.saturating_add(5).min(100),
        }
    }

    pub fn adjust_left(&mut self) {
        let opts = self.visible_options();
        let opt = opts[self.selected];
        match opt {
            MenuOption::Mode => {
                self.mode = self.mode.prev();
                let new_count = self.visible_options().len();
                if self.selected >= new_count {
                    self.selected = new_count - 1;
                }
            }
            MenuOption::Scope => self.scope = self.scope.toggle(),
            MenuOption::Spread => {
                self.spread = ((self.spread - 0.1) * 10.0).round() / 10.0;
                if self.spread < 0.1 { self.spread = 0.1; }
            }
            MenuOption::LightR => self.light_r = self.light_r.saturating_sub(5),
            MenuOption::LightG => self.light_g = self.light_g.saturating_sub(5),
            MenuOption::LightB => self.light_b = self.light_b.saturating_sub(5),
            MenuOption::DarkR => self.dark_r = self.dark_r.saturating_sub(5),
            MenuOption::DarkG => self.dark_g = self.dark_g.saturating_sub(5),
            MenuOption::DarkB => self.dark_b = self.dark_b.saturating_sub(5),
            MenuOption::PosterizeLevels => self.posterize_levels = self.posterize_levels.saturating_sub(1).max(2),
            MenuOption::EdgeThreshold => self.edge_threshold = self.edge_threshold.saturating_sub(5).max(5),
        }
    }

    /// Build the DitherParams from current menu state.
    pub fn dither_params(&self) -> crate::render::post_process::DitherParams {
        crate::render::post_process::DitherParams {
            brightness_boost: self.spread,
            color_light: (self.light_r, self.light_g, self.light_b),
            color_dark: (self.dark_r, self.dark_g, self.dark_b),
        }
    }

    /// Build the MoebiusParams from current menu state.
    pub fn moebius_params(&self) -> crate::render::post_process::MoebiusParams {
        crate::render::post_process::MoebiusParams {
            posterize_levels: self.posterize_levels,
            edge_threshold: self.edge_threshold,
        }
    }

    /// Draw the menu overlay.
    pub fn draw(&self, canvas: &mut Canvas<Window>, text: &mut TextRenderer) {
        if !self.visible {
            return;
        }

        let opts = self.visible_options();
        let labels: Vec<String> = opts.iter().map(|&o| self.option_label(o)).collect();

        let font_size = 16;
        let line_height = 22;
        let padding = 12;
        let menu_x = 10;
        let menu_y = 10;
        let menu_w = 340;
        let menu_h = padding * 2 + line_height * labels.len() as i32 + 24;

        // Semi-transparent background
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(10, 10, 25, 210));
        let bg = Rect::new(menu_x, menu_y, menu_w as u32, menu_h as u32);
        let _ = canvas.fill_rect(bg);

        // Border
        canvas.set_draw_color(Color::RGB(120, 100, 70));
        let _ = canvas.draw_rect(bg);

        // Title
        if let Some(tex) = text.render("[F1] Config", font_size, Color::RGB(150, 130, 90)) {
            let q = tex.query();
            let dst = Rect::new(menu_x + padding, menu_y + padding, q.width, q.height);
            let _ = canvas.copy(tex, None, dst);
        }

        // Options
        for (i, label) in labels.iter().enumerate() {
            let is_selected = i == self.selected;
            let prefix = if is_selected { "> " } else { "  " };
            let display = format!("{prefix}{label}");

            let color = if is_selected {
                Color::RGB(255, 220, 80)
            } else {
                Color::RGB(200, 190, 170)
            };

            let y = menu_y + padding + 24 + i as i32 * line_height;
            if let Some(tex) = text.render(&display, font_size, color) {
                let q = tex.query();
                let dst = Rect::new(menu_x + padding, y, q.width, q.height);
                let _ = canvas.copy(tex, None, dst);
            }
        }
    }
}
