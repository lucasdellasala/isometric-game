use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::config;
use crate::render::post_process::{ApplyScope, PostProcessMode};
use crate::render::text::TextRenderer;

/// Which submenu is active, or None for the top-level list.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveSubmenu {
    TopLevel,
    PostProcess,
    SpriteOffset,
    TilePreview,
    GameSettings,
    EntityScale,
}

/// A single adjustable value in a submenu.
struct MenuItem {
    label: &'static str,
}

/// Unified debug menu. Toggle with F1. Navigate with arrows, Enter, Escape.
pub struct DebugMenu {
    pub visible: bool,
    submenu: ActiveSubmenu,
    selected: usize,

    // --- Post-process settings ---
    pub pp_mode: PostProcessMode,
    pub pp_scope: ApplyScope,
    pub pp_spread: f64,
    pub pp_light: (u8, u8, u8),
    pub pp_dark: (u8, u8, u8),
    pub pp_posterize: u8,
    pub pp_edge_threshold: u8,

    // --- Sprite offset settings ---
    pub sprite_base_x: i32,
    pub sprite_base_y: i32,
    pub sprite_per_dir: bool,
    pub sprite_per_dir_offsets: [(i32, i32); 8],

    // --- Tile preview settings ---
    pub water_variant: u32,

    // --- Game settings ---
    pub fov_radius: i32,
    pub camera_zoom: f64,
    pub show_pathfinding: bool,

    // --- Entity scale settings ---
    pub entity_base_scale: f64,
    pub scale_player: f64,
    pub scale_npc: f64,
    pub scale_enemy_orc: f64,
    pub shadow_scale: f64,
    pub shadow_offset_y: i32,

    // --- Animation speed settings ---
    pub ticks_per_walk_frame: u32,
    pub ticks_per_idle_frame: u32,
}

impl DebugMenu {
    pub fn new(sprite_base_x: i32, sprite_base_y: i32) -> DebugMenu {
        DebugMenu {
            visible: false,
            submenu: ActiveSubmenu::TopLevel,
            selected: 0,

            pp_mode: PostProcessMode::Off,
            pp_scope: ApplyScope::TilesOnly,
            pp_spread: 0.5,
            pp_light: (250, 232, 205),
            pp_dark: (35, 25, 45),
            pp_posterize: 4,
            pp_edge_threshold: 30,

            sprite_base_x,
            sprite_base_y,
            sprite_per_dir: false,
            sprite_per_dir_offsets: [(0, 0); 8],

            water_variant: 17,

            fov_radius: config::DEFAULT_FOV_RADIUS,
            camera_zoom: config::DEFAULT_CAMERA_ZOOM,
            show_pathfinding: false,

            entity_base_scale: config::ENTITY_SCALE,
            scale_player: config::SCALE_PLAYER,
            scale_npc: config::SCALE_NPC,
            scale_enemy_orc: config::SCALE_ENEMY_ORC,
            shadow_scale: config::SHADOW_SCALE,
            shadow_offset_y: config::SHADOW_OFFSET_Y,

            ticks_per_walk_frame: config::TICKS_PER_ANIM_FRAME,
            ticks_per_idle_frame: config::TICKS_PER_IDLE_ANIM_FRAME,
        }
    }

    /// Export all debug settings to a JSON file.
    /// Called automatically when the game closes.
    pub fn export_json(&self, path: &str) {
        let json = format!(
            r#"{{
  "post_process": {{
    "mode": "{}",
    "scope": "{}",
    "spread": {:.1},
    "light_r": {}, "light_g": {}, "light_b": {},
    "dark_r": {}, "dark_g": {}, "dark_b": {},
    "posterize": {},
    "edge_threshold": {}
  }},
  "sprite_offset": {{
    "base_x": {},
    "base_y": {}
  }},
  "tile_preview": {{
    "water_variant": {}
  }},
  "game_settings": {{
    "fov_radius": {},
    "camera_zoom": {:.1},
    "show_pathfinding": {}
  }},
  "entity_scale": {{
    "base_scale": {:.2},
    "player": {:.2},
    "npc": {:.2},
    "enemy_orc": {:.2},
    "shadow_scale": {:.2},
    "shadow_offset_y": {}
  }},
  "animation": {{
    "ticks_per_walk_frame": {},
    "ticks_per_idle_frame": {}
  }}
}}"#,
            self.pp_mode.label(), self.pp_scope.label(),
            self.pp_spread,
            self.pp_light.0, self.pp_light.1, self.pp_light.2,
            self.pp_dark.0, self.pp_dark.1, self.pp_dark.2,
            self.pp_posterize, self.pp_edge_threshold,
            self.sprite_base_x, self.sprite_base_y,
            self.water_variant,
            self.fov_radius, self.camera_zoom, self.show_pathfinding,
            self.entity_base_scale, self.scale_player, self.scale_npc,
            self.scale_enemy_orc, self.shadow_scale, self.shadow_offset_y,
            self.ticks_per_walk_frame, self.ticks_per_idle_frame,
        );

        if let Err(e) = std::fs::write(path, &json) {
            eprintln!("Failed to export debug settings: {e}");
        } else {
            println!("Debug settings exported to {path}");
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if !self.visible {
            self.submenu = ActiveSubmenu::TopLevel;
            self.selected = 0;
        }
    }

    /// Get sprite offset for a facing direction.
    pub fn get_sprite_offset(&self, facing: crate::core::entity::Direction) -> (i32, i32) {
        let dir_idx = facing.spritesheet_frame() as usize;
        let (dx, dy) = if dir_idx < 8 { self.sprite_per_dir_offsets[dir_idx] } else { (0, 0) };
        (self.sprite_base_x + dx, self.sprite_base_y + dy)
    }

    pub fn dither_params(&self) -> crate::render::post_process::DitherParams {
        crate::render::post_process::DitherParams {
            brightness_boost: self.pp_spread,
            color_light: self.pp_light,
            color_dark: self.pp_dark,
        }
    }

    pub fn moebius_params(&self) -> crate::render::post_process::MoebiusParams {
        crate::render::post_process::MoebiusParams {
            posterize_levels: self.pp_posterize,
            edge_threshold: self.pp_edge_threshold,
        }
    }

    fn top_level_items(&self) -> Vec<&'static str> {
        vec![
            "Post-Process Effects",
            "Sprite Offset Adjust",
            "Tile Preview",
            "Game Settings",
            "Entity Scale",
        ]
    }

    fn submenu_items(&self) -> Vec<String> {
        match self.submenu {
            ActiveSubmenu::PostProcess => vec![
                format!("Mode:             {}", self.pp_mode.label()),
                format!("Apply to:         {}", self.pp_scope.label()),
                format!("Spread:           {:.1}", self.pp_spread),
                format!("Light R:          {}", self.pp_light.0),
                format!("Light G:          {}", self.pp_light.1),
                format!("Light B:          {}", self.pp_light.2),
                format!("Dark R:           {}", self.pp_dark.0),
                format!("Dark G:           {}", self.pp_dark.1),
                format!("Dark B:           {}", self.pp_dark.2),
                format!("Posterize levels: {}", self.pp_posterize),
                format!("Edge threshold:   {}", self.pp_edge_threshold),
            ],
            ActiveSubmenu::SpriteOffset => {
                let mut items = vec![
                    format!("Base offset X:    {}", self.sprite_base_x),
                    format!("Base offset Y:    {}", self.sprite_base_y),
                    format!("Per-direction:    {}", if self.sprite_per_dir { "ON" } else { "OFF" }),
                ];
                if self.sprite_per_dir {
                    for i in 0..8u16 {
                        let angle = i * 45;
                        let (dx, dy) = self.sprite_per_dir_offsets[i as usize];
                        items.push(format!("  {:03}° offset:    ({}, {})", angle, dx, dy));
                    }
                }
                items
            }
            ActiveSubmenu::TilePreview => vec![
                format!("Water variant:    {:02}/18", self.water_variant),
            ],
            ActiveSubmenu::GameSettings => vec![
                format!("FOV radius:       {}", self.fov_radius),
                format!("Camera zoom:      {:.1}", self.camera_zoom),
                format!("Show pathfinding: {}", if self.show_pathfinding { "ON" } else { "OFF" }),
            ],
            ActiveSubmenu::EntityScale => vec![
                format!("Base scale:       {:.2}", self.entity_base_scale),
                format!("Player scale:     {:.2}", self.scale_player),
                format!("NPC scale:        {:.2}", self.scale_npc),
                format!("Orc scale:        {:.2}", self.scale_enemy_orc),
                format!("Shadow scale:     {:.2}", self.shadow_scale),
                format!("Shadow offset Y:  {}", self.shadow_offset_y),
                format!("Walk anim speed:  {} ticks/frame", self.ticks_per_walk_frame),
                format!("Idle anim speed:  {} ticks/frame", self.ticks_per_idle_frame),
            ],
            ActiveSubmenu::TopLevel => vec![],
        }
    }

    fn item_count(&self) -> usize {
        match self.submenu {
            ActiveSubmenu::TopLevel => self.top_level_items().len(),
            _ => self.submenu_items().len(),
        }
    }

    pub fn handle_up(&mut self) {
        let count = self.item_count();
        if count == 0 { return; }
        self.selected = if self.selected > 0 { self.selected - 1 } else { count - 1 };
    }

    pub fn handle_down(&mut self) {
        let count = self.item_count();
        if count == 0 { return; }
        self.selected = if self.selected < count - 1 { self.selected + 1 } else { 0 };
    }

    pub fn handle_enter(&mut self) {
        if self.submenu == ActiveSubmenu::TopLevel {
            self.submenu = match self.selected {
                0 => ActiveSubmenu::PostProcess,
                1 => ActiveSubmenu::SpriteOffset,
                2 => ActiveSubmenu::TilePreview,
                3 => ActiveSubmenu::GameSettings,
                4 => ActiveSubmenu::EntityScale,
                _ => ActiveSubmenu::TopLevel,
            };
            self.selected = 0;
        }
    }

    pub fn handle_back(&mut self) {
        if self.submenu != ActiveSubmenu::TopLevel {
            self.submenu = ActiveSubmenu::TopLevel;
            self.selected = 0;
        } else {
            self.visible = false;
        }
    }

    pub fn handle_left(&mut self, player_facing: crate::core::entity::Direction) {
        match self.submenu {
            ActiveSubmenu::PostProcess => match self.selected {
                0 => self.pp_mode = self.pp_mode.prev(),
                1 => self.pp_scope = self.pp_scope.toggle(),
                2 => { self.pp_spread = ((self.pp_spread - 0.1) * 10.0).round() / 10.0; if self.pp_spread < 0.1 { self.pp_spread = 0.1; } }
                3 => self.pp_light.0 = self.pp_light.0.saturating_sub(5),
                4 => self.pp_light.1 = self.pp_light.1.saturating_sub(5),
                5 => self.pp_light.2 = self.pp_light.2.saturating_sub(5),
                6 => self.pp_dark.0 = self.pp_dark.0.saturating_sub(5),
                7 => self.pp_dark.1 = self.pp_dark.1.saturating_sub(5),
                8 => self.pp_dark.2 = self.pp_dark.2.saturating_sub(5),
                9 => self.pp_posterize = self.pp_posterize.saturating_sub(1).max(2),
                10 => self.pp_edge_threshold = self.pp_edge_threshold.saturating_sub(5).max(5),
                _ => {}
            },
            ActiveSubmenu::SpriteOffset => match self.selected {
                0 => self.sprite_base_x -= 1,
                1 => self.sprite_base_y -= 1,
                2 => self.sprite_per_dir = !self.sprite_per_dir,
                n if n >= 3 && self.sprite_per_dir => {
                    let idx = n - 3;
                    if idx < 8 { self.sprite_per_dir_offsets[idx].0 -= 1; }
                }
                _ => {}
            },
            ActiveSubmenu::TilePreview => match self.selected {
                0 => self.water_variant = if self.water_variant <= 1 { 18 } else { self.water_variant - 1 },
                _ => {}
            },
            ActiveSubmenu::GameSettings => match self.selected {
                0 => self.fov_radius = (self.fov_radius - 1).max(5),
                1 => { self.camera_zoom = ((self.camera_zoom - 0.1) * 10.0).round() / 10.0; if self.camera_zoom < 0.5 { self.camera_zoom = 0.5; } }
                2 => self.show_pathfinding = !self.show_pathfinding,
                _ => {}
            },
            ActiveSubmenu::EntityScale => match self.selected {
                0 => self.entity_base_scale = ((self.entity_base_scale - 0.01) * 100.0).round() / 100.0,
                1 => self.scale_player = ((self.scale_player - 0.05) * 100.0).round() / 100.0,
                2 => self.scale_npc = ((self.scale_npc - 0.05) * 100.0).round() / 100.0,
                3 => self.scale_enemy_orc = ((self.scale_enemy_orc - 0.05) * 100.0).round() / 100.0,
                4 => self.shadow_scale = ((self.shadow_scale - 0.05) * 100.0).round() / 100.0,
                5 => self.shadow_offset_y -= 1,
                6 => self.ticks_per_walk_frame = self.ticks_per_walk_frame.saturating_sub(1).max(1),
                7 => self.ticks_per_idle_frame = self.ticks_per_idle_frame.saturating_sub(1).max(1),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_right(&mut self, player_facing: crate::core::entity::Direction) {
        match self.submenu {
            ActiveSubmenu::PostProcess => match self.selected {
                0 => self.pp_mode = self.pp_mode.next(),
                1 => self.pp_scope = self.pp_scope.toggle(),
                2 => self.pp_spread = (self.pp_spread + 0.1).min(3.0),
                3 => self.pp_light.0 = self.pp_light.0.saturating_add(5),
                4 => self.pp_light.1 = self.pp_light.1.saturating_add(5),
                5 => self.pp_light.2 = self.pp_light.2.saturating_add(5),
                6 => self.pp_dark.0 = self.pp_dark.0.saturating_add(5),
                7 => self.pp_dark.1 = self.pp_dark.1.saturating_add(5),
                8 => self.pp_dark.2 = self.pp_dark.2.saturating_add(5),
                9 => self.pp_posterize = (self.pp_posterize + 1).min(8),
                10 => self.pp_edge_threshold = self.pp_edge_threshold.saturating_add(5).min(100),
                _ => {}
            },
            ActiveSubmenu::SpriteOffset => match self.selected {
                0 => self.sprite_base_x += 1,
                1 => self.sprite_base_y += 1,
                2 => self.sprite_per_dir = !self.sprite_per_dir,
                n if n >= 3 && self.sprite_per_dir => {
                    let idx = n - 3;
                    if idx < 8 { self.sprite_per_dir_offsets[idx].0 += 1; }
                }
                _ => {}
            },
            ActiveSubmenu::TilePreview => match self.selected {
                0 => self.water_variant = if self.water_variant >= 18 { 1 } else { self.water_variant + 1 },
                _ => {}
            },
            ActiveSubmenu::GameSettings => match self.selected {
                0 => self.fov_radius = (self.fov_radius + 1).min(40),
                1 => self.camera_zoom = (self.camera_zoom + 0.1).min(4.0),
                2 => self.show_pathfinding = !self.show_pathfinding,
                _ => {}
            },
            ActiveSubmenu::EntityScale => match self.selected {
                0 => self.entity_base_scale = ((self.entity_base_scale + 0.01) * 100.0).round() / 100.0,
                1 => self.scale_player = ((self.scale_player + 0.05) * 100.0).round() / 100.0,
                2 => self.scale_npc = ((self.scale_npc + 0.05) * 100.0).round() / 100.0,
                3 => self.scale_enemy_orc = ((self.scale_enemy_orc + 0.05) * 100.0).round() / 100.0,
                4 => self.shadow_scale = ((self.shadow_scale + 0.05) * 100.0).round() / 100.0,
                5 => self.shadow_offset_y += 1,
                6 => self.ticks_per_walk_frame += 1,
                7 => self.ticks_per_idle_frame += 1,
                _ => {}
            },
            _ => {}
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, text: &mut TextRenderer) {
        if !self.visible { return; }

        let font_size = 16;
        let line_height = 22;
        let padding = 12;
        let menu_x = 10;
        let menu_y = 10;
        let menu_w: u32 = 360;

        let (title, items): (&str, Vec<String>) = match self.submenu {
            ActiveSubmenu::TopLevel => {
                ("Debug Menu", self.top_level_items().iter().map(|s| s.to_string()).collect())
            }
            ActiveSubmenu::PostProcess => ("Post-Process Effects", self.submenu_items()),
            ActiveSubmenu::SpriteOffset => ("Sprite Offset Adjust", self.submenu_items()),
            ActiveSubmenu::TilePreview => ("Tile Preview", self.submenu_items()),
            ActiveSubmenu::GameSettings => ("Game Settings", self.submenu_items()),
            ActiveSubmenu::EntityScale => ("Entity Scale", self.submenu_items()),
        };

        let menu_h = (padding * 2 + line_height * (items.len() as i32 + 1) + 28) as u32;

        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(10, 10, 25, 220));
        let bg = Rect::new(menu_x, menu_y, menu_w, menu_h);
        let _ = canvas.fill_rect(bg);
        canvas.set_draw_color(Color::RGB(120, 100, 70));
        let _ = canvas.draw_rect(bg);

        // Title + hint
        let hint = if self.submenu == ActiveSubmenu::TopLevel {
            "[TAB] close  [Enter] select"
        } else {
            "[Esc] back  [Left/Right] adjust"
        };
        if let Some(tex) = text.render(&format!("{title}  —  {hint}"), 12, Color::RGB(150, 130, 90)) {
            let q = tex.query();
            let dst = Rect::new(menu_x + padding, menu_y + padding, q.width, q.height);
            let _ = canvas.copy(tex, None, dst);
        }

        for (i, label) in items.iter().enumerate() {
            let is_sel = i == self.selected;
            let prefix = if is_sel { "> " } else { "  " };
            let color = if is_sel { Color::RGB(255, 220, 80) } else { Color::RGB(200, 190, 170) };
            let display = format!("{prefix}{label}");

            let y = menu_y + padding + 26 + i as i32 * line_height;
            if let Some(tex) = text.render(&display, font_size, color) {
                let q = tex.query();
                let dst = Rect::new(menu_x + padding, y, q.width, q.height);
                let _ = canvas.copy(tex, None, dst);
            }
        }
    }
}
