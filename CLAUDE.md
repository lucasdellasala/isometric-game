# CLAUDE.md — Multiplayer Isometric D&D RPG in Rust

## Project context

Multiplayer co-op D&D-style RPG with isometric graphics (Fallout 1 aesthetic). Built from scratch in Rust with SDL2, no game engine. Developer learned Rust from JS/TS and completed the engine foundation. Now evolving it into a full game with 10 phases.

---

## Developer profile

- **Background:** JavaScript, TypeScript, Node.js
- **Rust level:** Intermediate — knows ownership, borrowing, structs, enums, traits, generics, closures, iterators, modules, serde. Now also knows lifetimes (`'a`) from P2 (SDL2 textures tied to TextureCreator).
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

### Current code architecture (post-P1, post-P2)

```
main.rs (thin game loop — ~130 lines)
  │
  ├─ Creates: GameState, AssetManager, Camera (client-only)
  │
  ├─ INPUT: SDL2 events → translated to GameInput enum
  │   ├─ Mouse click → iso::screen_to_grid() → GameInput::MoveTo
  │   ├─ WASD held → GameInput::MoveDirection
  │   └─ Escape → quit
  │
  ├─ UPDATE (fixed timestep, 60 ticks/sec):
  │   ├─ state.apply_input(GameInput) → processes movement/pathfinding
  │   ├─ state.tick() → advances entities, updates FOV, returns Vec<GameEvent>
  │   └─ camera.x/y = local_player.visual_x/y (smooth follow)
  │
  └─ RENDER:
      └─ renderer::draw_world(&canvas, &GameState, &Camera, &mut AssetManager)
          ├─ For each tile (row by row, back to front):
          │   ├─ Frustum cull → skip off-screen tiles
          │   ├─ FOV brightness → texture.set_color_mod() for darkening
          │   └─ canvas.copy() tile texture (PNG or generated placeholder)
          ├─ Draw click target marker
          └─ Draw all entities on top (currently no depth interleaving — fix in P3)
```

### Key technical decisions
- **Scancode vs Keycode:** Using `Scancode` for movement (physical key position, layout-independent)
- **FOV uses distance falloff, not lerp:** Lerp-based FOV transition caused motion sickness. Distance falloff (100% brightness in inner 50% of radius, linear fade to edge) is instant but smooth
- **Entities draw on top of everything:** Known issue — proper depth interleaving caused flickering due to visual interpolation. Fix in P3 with row-by-row entity sorting
- **Frustum culling:** Only draw tiles within screen bounds + 64px margin. 200x200 map at 76fps
- **SDL2_image not used:** The `sdl2` crate's `image` feature requires precompiled SDL2_image lib. Instead, we use the `image` crate (pure Rust) to decode PNGs and create SDL2 textures from surfaces
- **Texture darkening:** Using `texture.set_color_mod(b, b, b)` instead of overlay rectangles — overlays leaked into transparent areas of diamond tiles
- **Wall rendering:** Currently uses line-drawn parallelogram side faces + textured top. Full pre-rendered wall sprites (64x64) are loaded when available but need rotation/direction fixes (P3)

### Source files

| File | Lines | What it does |
|------|-------|-------------|
| `main.rs` | ~130 | SDL2 init, thin game loop: poll events → GameInput → apply → tick → render. Creates AssetManager with TextureCreator lifetime |
| `game_state.rs` | ~110 | `GameState` struct owns: tilemap, `Vec<Entity>`, fov_map, click_target. Methods: `apply_input()`, `tick()`, `spawn_entity()`, `local_player()`. Spawns player at (0,0) and test NPC at (4,3) |
| `entity.rs` | ~130 | `Entity` struct with `id: u64`, `EntityKind` (Player/Npc/Enemy), grid + visual position, pathfinding queue, move cooldown. `try_move()`, `move_to()` (A*), `update()` (path advancement + lerp) |
| `input.rs` | ~17 | `GameInput` enum (MoveDirection, MoveTo) and `GameEvent` enum (EntityMoved, PathNotFound) — the client/server message boundary |
| `assets.rs` | ~170 | `AssetManager<'a>` with lifetime tied to `TextureCreator`. Generates placeholder diamond textures at startup. `load_image()` loads real PNGs via `image` crate → SDL2 surface → texture. Falls back to placeholders if files missing |
| `renderer.rs` | ~200 | `draw_world()` takes `&GameState` + `&mut AssetManager`. Tiles via `canvas.copy()` with `set_color_mod` for FOV. Walls: line-drawn side faces or full sprite. Entities drawn on top |
| `fov.rs` | ~170 | `FovMap::compute()`: 8-octant recursive shadowcasting. `brightness: Vec<f64>` per tile with distance falloff. Walls (height > 0) block vision |
| `pathfinding.rs` | ~130 | `find_path()`: A* with Manhattan heuristic, 4-directional, `BinaryHeap` (reversed for min-heap). Returns `Vec<Pos>` excluding start |
| `tilemap.rs` | ~90 | `Tilemap` from JSON via serde. `TileKind` enum: Grass/Dirt/Water/Wall. Each has `top_color()`, `side_color()`, `height()`. Flat `Vec` with `row * cols + col` indexing |
| `iso.rs` | ~20 | `grid_to_screen(x,y)→(sx,sy)` and `screen_to_grid(sx,sy)→(x,y)`. Tile: 64x32. Formula: `sx = (x-y)*32, sy = (x+y)*16` |
| `camera.rs` | ~12 | `Camera { x, y }` — viewport offset. Follows `player.visual_x/y` each tick |

### Assets structure
```
assets/
  map.json            — 32x32 test map (Grass, Dirt, Water, Wall)
  map_large.json      — 200x200 generated map (for perf testing)
  tiles/
    Ground/           — Real isometric tiles from Woulette's RPG tileset (itch.io)
      ground_stone.png      — 64x32 stone floor (used as Grass tile)
      ground_dungeon.png    — 64x32 dungeon floor (used as Dirt tile)
      wall_stone_left_64x32.png   — 64x64 wall with left face
      wall_stone_right_64x32.png  — 64x64 wall with right face
      (+ variants and cap pieces)
    Decor/            — Props (chests, bones, lanterns) — spritesheets, not yet integrated
  characters/         — Empty, awaiting character sprites
```

### Constants
- Tile size: 64x32 pixels (TILE_WIDTH, TILE_HEIGHT in iso.rs)
- Fixed timestep: 60 ticks/sec (TICK_DURATION in main.rs)
- Move cooldown: 6 ticks for WASD, 8 ticks between path steps
- FOV radius: 10 tiles
- Explored brightness: 0.35, falloff starts at 50% of radius
- Window: starts maximized, resizable. Renderer reads `canvas.output_size()` dynamically

---

## Roadmap — 10 Phases

| Phase | Goal | Status |
|-------|------|--------|
| **P1** | GameState architecture — entity system, input/event pattern | **Done** |
| **P2** | Real sprites — PNG textures, AssetManager with lifetimes | **Done** |
| **P3** | Multiple entities + depth fix — NPCs, interaction, text UI | **Next** |
| **P4** | Audio + polish — music, SFX, animated tiles, particles | Pending |
| **P5** | D&D stats + inventory — character sheet, dice, items | Pending |
| **P6** | Turn-based combat + AI — initiative, actions, attacks | Pending |
| **P7** | Map transitions + save/load — connected world, interiors | Pending |
| **P8** | Dialogue + story — branching trees, quests, flags | Pending |
| **P9** | Multiplayer — tokio, client-server, up to 6 players co-op | Pending |
| **P10** | Map editor — visual tool for content creation | Pending |

### Dependencies per phase
P1: none | P2: `image` crate | P3: sdl2 `ttf` | P4: sdl2 `mixer` | P5: `rand` | P6-P8: none | P9: `tokio`, `bincode` | P10: none

---

## Current Phase Detail — P3: Multiple Entities + Depth Sorting Fix

**Goal:** NPCs on the map with correct depth sorting (entities behind walls are occluded). Basic interaction system and text UI.

**What needs to happen:**

1. **Depth sorting fix** — Instead of drawing all tiles then all entities, interleave them by row. For each row: draw tiles, then draw entities whose visual position falls in that row. This makes entities go behind walls correctly. The previous attempt (using grid_y for draw order) caused flickering because visual_y interpolates between rows during movement. Need to compute effective visual row from the lerped position.

2. **Directional wall tiles** — Current `TileKind::Wall` is a single type. Need to either:
   - Add directional variants (`WallLeft`, `WallRight`, `WallTop`, etc.) to the JSON map format
   - Or auto-detect wall direction based on neighboring tiles
   - The tileset has separate `wall_stone_left` and `wall_stone_right` PNGs

3. **NPC data in map JSON** — Extend the map format to include entity spawn points:
   ```json
   { "cols": 32, "rows": 32, "tiles": [...], "entities": [
     { "kind": "Npc", "name": "Guide", "x": 4, "y": 3 }
   ]}
   ```

4. **Interaction system** — Press E when adjacent to NPC → triggers `GameInput::Interact { entity_id, target_id }` → `GameEvent::InteractionStarted`. For P3 this just shows a text box.

5. **Text UI with SDL2_ttf** — Add `sdl2` `ttf` feature. Render dialogue text in a box at the bottom of the screen. Need to handle the SDL2_ttf bundling (similar issue to SDL2_image — may need to use a Rust-native font renderer instead).

**Known issues to resolve:**
- Wall sprites are currently rotated/misaligned — the left wall sprite is used for all walls regardless of orientation
- Entity sprites are placeholder colored shapes (body + head) — will be replaced when character sprites are available
- The `Decor/` props are spritesheets (2432x1216) — would need sprite region extraction to use individual props

**Verification:** Entities correctly appear behind walls when walking past them. NPCs loaded from map JSON. Press E shows a text box with NPC name.

---

## Reference

**The Rust Book:** https://doc.rust-lang.org/stable/book/

**Rust concepts learned so far:**
Variables, types, functions, macros, control flow, ownership, borrowing, slices, structs, enums, Option, Result, traits, generics, closures, iterators, modules, serde, **lifetimes (`'a`)**

**Rust concepts not yet learned** (explain with JS/TS comparison before using):
Arc/Mutex → P9 | async/await → P9 | trait objects (`dyn Trait`) → TBD | `#[serde(skip)]` → P7
