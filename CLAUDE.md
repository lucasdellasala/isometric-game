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
- **Scancode for movement:** WASD only (no arrow keys — reserved for debug menu)
- **Text rendering:** rusttype crate (pure Rust), not SDL2_ttf (avoids native lib bundling)
- **SDL2_image not used:** image crate (pure Rust) decodes PNGs → SDL2 surfaces → textures
- **Spritesheet tiles:** Extracted from spritesheets to individual 128×64 PNGs with diamond transparency mask. AssetManager loads them as regular textures
- **NPC spritesheets:** 1024×256 PNGs with 8 directional frames (128×256 each). Renderer uses src_rect to select frame
- **Grass decorations:** Generated per-tile with deterministic LCG PRNG. Back tufts drawn with tiles (dithered), front tufts drawn after entities (occlude player feet)
- **Entity outlines:** Pre-computed at load time by scanning sprite PNGs for edge pixels (transparent with opaque neighbor). Stored as `Vec<(i32,i32)>` per frame. Drawn with `canvas.fill_rect()` for uniform color regardless of sprite content
- **Occlusion transparency:** Entities within Chebyshev distance ≤ 1 from player with higher depth row → alpha 128. Depth-row based, not pixel intersection (see IDEAS.md #2 for rationale)

### Assets structure
```
assets/
  fonts/default.ttf                    — UI font (Arial)
  maps/map.json                        — 64×64 test map with entities, walls
  sprites/
    player/
      idle/entity_player_000..315.png  — 8 directional idle sprites (128×256)
      walk/entity_player_walk_*        — 8 dirs × 8 frames = 64 walk sprites
    npc/
      entity_npc_*.png                 — 9 variant spritesheets (1024×256, 8 frames each)
    enemy/entity_enemy.png             — Enemy placeholder
    decorations/grass_tuft_01..08.png  — Grass tufts (16×24)
  tiles/
    forest/forest_01..18.png           — Forest ground tiles (128×64)
    water/water_01..18.png             — Water tiles
    terrain/terrain_01..18.png         — Terrain/dirt/stone tiles

assets_dev/                            — Development files (NOT committed)
  tiles_spritesheet/                   — Source spritesheets
  CelShading_old/                      — Old cel-shading assets
  (Blender files, FBX, mesh bundles)
```

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
