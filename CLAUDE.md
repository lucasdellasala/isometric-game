# CLAUDE.md — Multiplayer Isometric D&D RPG in Rust

## Project context

Multiplayer co-op D&D-style RPG with isometric graphics. Built from scratch in Rust with SDL2, no game engine. Developer learned Rust from JS/TS and completed the engine foundation. Now evolving it into a full game with 10 phases.

---

## Developer profile

- **Background:** JavaScript, TypeScript, Node.js
- **Rust level:** Intermediate — knows ownership, borrowing, structs, enums, traits, generics, closures, iterators, modules, serde, lifetimes (`'a`), rand.
- **IDE:** Zed (rust-analyzer built-in)
- **Environment:** Windows 10, Rust 1.94.1, CMake (Visual Studio 16 2019 generator)
- **Goal:** Understand every line — not just make it work.

### Rules
- Always respond in Spanish
- Compare new Rust concepts with JS/TS equivalents
- Explain the "why", not just the "how"
- Prefer explicit code over idiomatic until both forms are known
- When code doesn't compile, help read the error before giving the solution
- If using a concept not yet learned, stop and explain first
- **Never hardcode visual/gameplay values inline.** Any numeric value that controls appearance or behavior (colors, sizes, speeds, thresholds, offsets) must be a named `const` at the top of the file or module. Examples: `const OUTLINE_THICKNESS: i32 = 4;`, `const NPC_HIGHLIGHT_COLOR: Color = Color::RGB(100, 255, 100);`. This makes tuning easy without searching through logic code.

---

## Architecture

### The Golden Rule
```
GameInput → GameState.apply_input() → GameState.tick() → Vec<GameEvent>
```
- `GameState` is pure logic — never knows about SDL2, textures, audio, or network
- Renderer only reads `&GameState` (immutable)
- Audio/VFX react to `GameEvent` values
- This IS the client-server boundary — networking (P9) becomes just transport

### Module structure (core/render/ui)

```
src/
  main.rs                    — SDL2 init, game loop, input dispatch
  config.rs                  — ALL tunable constants (colors, sizes, speeds, thresholds)
  core/                      — Pure game logic (no SDL2)
    mod.rs
    input.rs                 — GameInput, GameEvent enums
    tilemap.rs               — TileKind (Grass/Dirt/Stone/Water), WallObject, Tilemap from JSON
    entity.rs                — Entity, EntityKind, NpcVariant, facing direction, walk animation
    pathfinding.rs           — A* with Manhattan heuristic
    fov.rs                   — 8-octant recursive shadowcasting
    game_state.rs            — GameState, ActiveDialogue, entity spawning
  render/                    — All rendering (SDL2-dependent)
    mod.rs
    iso.rs                   — Isometric projection (re-exports TILE_WIDTH/HEIGHT from config)
    camera.rs                — Camera viewport offset
    assets.rs                — AssetManager (textures, spritesheets)
    text.rs                  — TextRenderer (rusttype, pure Rust)
    renderer.rs              — draw_tiles, draw_entities_and_ui, render_frame
    post_process.rs          — Dithering (Bayer 4x4 palette), Moebius (posterize + Sobel)
    decorations.rs           — Grass tufts with pseudo-random placement
  ui/                        — UI systems
    mod.rs
    debug_menu.rs            — Unified debug menu (TAB key)
```

### Key technical decisions
- **Tile size:** 128×64 pixels (TILE_WIDTH, TILE_HEIGHT in iso.rs)
- **Camera zoom:** Configurable via debug menu (default 1.6), applied at render time
- **FOV radius:** Configurable via GameState.fov_radius (default 18)
- **Depth sorting:** By isometric depth row (col + row). Entity depth uses max(grid_depth, visual_depth_ceil) to avoid flickering during movement
- **8-directional movement:** WASD for 4 grid-cardinal directions + combos (W+D, D+S, S+A, A+W) for diagonals. Arrow keys reserved for debug menu
- **Direction system:** `Direction` enum with 8 screen-cardinals (N, NE, E, SE, S, SW, W, NW). Used for facing, sprite selection, and pathfinding
- **Pathfinding:** A* with 8 directions (cardinal cost 10, diagonal cost 14 ≈ √2×10), octile distance heuristic
- **Text rendering:** rusttype crate (pure Rust), not SDL2_ttf (avoids native lib bundling)
- **SDL2_image not used:** image crate (pure Rust) decodes PNGs → SDL2 surfaces → textures
- **Spritesheet tiles:** Extracted from spritesheets to individual 128×64 PNGs with diamond transparency mask. AssetManager loads them as regular textures
- **All entity sprites are individual PNGs:** Player, NPCs, and enemies all use one PNG per direction (128×256). No spritesheets at render time — no src_rect. NPC naming: `entity_npc_{variant}_{DIR}.png`. Enemy: `entity_enemy_{DIR}.png`
- **Grass decorations:** Generated per-tile with deterministic LCG PRNG. Back tufts drawn with tiles (dithered), front tufts drawn after entities (occlude player feet)
- **Entity outlines:** Pre-computed at load time by `generate_outline_for_image()` — scans each PNG for outer edge pixels (transparent with opaque neighbor). Stored as `Vec<(i32,i32)>` per direction. Drawn with `canvas.fill_rect()` for uniform color
- **Occlusion transparency:** Entities within Chebyshev distance ≤ 1 from player with higher depth row → alpha 128. Depth-row based, not pixel intersection (see IDEAS.md #2 for rationale)
- **NPC interaction:** Chebyshev distance ≤ 1 for interaction range (8 directions + same tile). Outline highlight + `[E] Hablar` prompt

### Assets structure
```
assets/
  fonts/default.ttf                    — UI font (Arial)
  maps/map.json                        — 64×64 test map with entities, walls
  sprites/
    player/
      idle/entity_player_S..SE.png     — 8 directional idle sprites (128×256)
      walk/entity_player_walk_*        — 8 dirs × 8 frames = 64 walk sprites
    npc/
      entity_npc_*.png                 — 7 variant spritesheets (1024×256, 8 frames each)
    enemy/
      entity_enemy.png                 — Default enemy spritesheet (1024×256)
      entity_npc_orc.png               — Orc enemy spritesheet (1024×256)
    decorations/grass_tuft_01..08.png  — Grass tufts (16×24)
  tiles/
    forest/forest_01..18.png           — Forest ground tiles (128×64)
    water/water_01..18.png             — Water tiles
    terrain/terrain_01..18.png         — Terrain/dirt/stone tiles

assets_dev/                            — Development files (NOT committed)
  tiles_spritesheet/                   — Source spritesheets
  (Blender files, FBX, mesh bundles)
```

### Asset specifications

All character sprites (player, NPC, enemy) share the same camera and format:

**Camera setup:**
- Orthographic isometric projection
- Camera rotation: X=60°, Z=45° (fixed, never changes)
- The character rotates on Z axis; the camera stays still

**Player idle sprites:**
- Size: 128×256 px, RGBA with transparent background
- 8 individual PNGs, one per facing direction
- Naming: `entity_player_{cardinal}.png` where cardinal = S, SW, W, NW, N, NE, E, SE
- `_S` = character facing the camera (front visible, screen down)
- `_N` = character facing away from camera (back visible, screen up)
- `_W` = character facing screen-left
- `_E` = character facing screen-right
- `_SW` = screen down-left, `_SE` = screen down-right, `_NW` = screen up-left, `_NE` = screen up-right
- Location: `assets/sprites/player/idle/`

**Player walk sprites:**
- Same size and camera as idle (128×256 px RGBA)
- 8 directions × 8 frames = 64 PNGs
- Naming: `entity_player_walk_{cardinal}_{frame}.png` where cardinal = S, SW, W, NW, N, NE, E, SE and frame = 0..7
- Example: `entity_player_walk_S_0.png` ... `entity_player_walk_S_7.png`, `entity_player_walk_SW_0.png` ... etc.
- One full walk cycle per direction
- Location: `assets/sprites/player/walk/`

**NPC sprites (individual PNGs per direction, same as player):**
- Size: 128×256 px, RGBA with transparent background
- 8 PNGs per variant, one per direction
- Naming: `entity_npc_{ethnicity}_{clothes}_{hair}_{DIR}.png`
  - Ethnicity: african, caucasian, latino
  - Colors: bk=black, bn=brown, cr=cream, gn=green, yl=yellow
  - DIR: S, SW, W, NW, N, NE, E, SE
  - Example: `entity_npc_african_cr_bk_S.png`, `entity_npc_african_cr_bk_NE.png`
- Location: `assets/sprites/npc/`

**Enemy sprites (individual PNGs per direction, same as player):**
- Size: 128×256 px, RGBA with transparent background
- 8 PNGs per variant
- Naming: `entity_enemy_{DIR}.png`, `entity_npc_orc_{DIR}.png`
- Location: `assets/sprites/enemy/`

**Ground tiles:**
- Size: 128×64 px, RGBA with transparent background
- Diamond (rhombus) shape, pixels outside the diamond are transparent
- Ratio: exactly 2:1 (width = 2× height)
- Naming: `{type}_{number:02}.png` (e.g., `forest_04.png`, `water_17.png`)
- Location: `assets/tiles/{type}/` (forest, water, terrain)

**Grass decorations:**
- Size: 16×24 px, RGBA with transparent background
- 3-5 procedural grass blades per sprite, anchored at bottom-center
- 8 variants: `grass_tuft_01.png` to `grass_tuft_08.png`
- Location: `assets/sprites/decorations/`

### Constants
All tunable values live in `src/config.rs`. Categories:
- **Tile & Projection:** TILE_WIDTH, TILE_HEIGHT
- **Camera:** DEFAULT_CAMERA_ZOOM
- **Entity Rendering:** ENTITY_OFFSET_X/Y, ENTITY_SCALE
- **Entity Behavior:** LERP_SPEED, PATH_STEP_TICKS, WALK_ANIM_FRAMES, TICKS_PER_ANIM_FRAME, IDLE_ROTATE_MIN/MAX_TICKS, MOVE_COOLDOWN
- **FOV & Visibility:** DEFAULT_FOV_RADIUS, EXPLORED_BRIGHTNESS
- **Interaction Highlight:** HIGHLIGHT_OUTLINE_PX, HIGHLIGHT_COLOR_NPC/ENEMY, HIGHLIGHT_ALPHA_ADJACENT/HOVER, HIGHLIGHT_PROMPT_*
- **Hover & Markers:** HOVER_COLOR, MARKER_COLOR, MARKER_SIZE_BASE
- **Dialogue Box:** DIALOGUE_BOX_HEIGHT/MARGIN/PADDING, DIALOGUE_BG/BORDER/NAME/TEXT/HINT colors and font sizes
- **Frustum Culling:** CULL_MARGIN
- **Wall Cube:** WALL_LEFT/RIGHT_BRIGHTNESS, WALL_LEFT/RIGHT_COLOR
- Window: starts maximized, resizable. Fixed timestep: 60 ticks/sec (TICK_DURATION in main.rs)

---

## Roadmap — 10 Phases

| Phase | Goal | Status |
|-------|------|--------|
| **P1** | GameState architecture — entity system, input/event pattern | **Done** |
| **P2** | Real sprites — PNG textures, AssetManager with lifetimes | **Done** |
| **P3** | Multiple entities + depth fix — NPCs, interaction, text UI | **Done** |
| **P4** | Audio + polish — music, SFX, animated tiles, particles | Pending |
| **P5** | D&D stats + inventory — character sheet, dice, items | Pending |
| **P6** | Turn-based combat + AI — initiative, actions, attacks | Pending |
| **P7** | Map transitions + save/load — connected world, interiors | Pending |
| **P8** | Dialogue + story — branching trees, quests, flags | Pending |
| **P9** | Multiplayer — tokio, client-server, up to 6 players co-op | Pending |
| **P10** | Map editor — visual tool for content creation | Pending |

### Dependencies
P1: none | P2: `image` | P3: `rusttype`, `rand` | P4: sdl2 `mixer` | P5: `rand` | P6-P8: none | P9: `tokio`, `bincode` | P10: none

---

## Debug menu (TAB key)

Unified debug menu with submenus navigated by arrow keys + Enter/Escape:

- **Post-Process Effects:** Mode (Off/Dithering/Moebius), Scope, Spread, Light/Dark RGB, Posterize, Edge threshold
- **Sprite Offset Adjust:** Base X/Y, per-direction toggle, offsets per angle
- **Tile Preview:** Water variant (01-18)
- **Game Settings:** FOV radius (5-40), Camera zoom (0.5-4.0)

---

## Reference

**The Rust Book:** https://doc.rust-lang.org/stable/book/

**Rust concepts learned so far:**
Variables, types, functions, macros, control flow, ownership, borrowing, slices, structs, enums, Option, Result, traits, generics, closures, iterators, modules, serde, lifetimes (`'a`), `rand`, HashMap, `wrapping_mul`, LCG PRNG

**Rust concepts not yet learned** (explain with JS/TS comparison before using):
Arc/Mutex → P9 | async/await → P9 | trait objects (`dyn Trait`) → TBD | `#[serde(skip)]` → P7
