# CLAUDE.md — Multiplayer Isometric D&D RPG in Rust

## Project context

Multiplayer co-op D&D-style RPG with isometric graphics (Fallout 1 aesthetic). Built from scratch in Rust with SDL2, no game engine. Developer learned Rust from JS/TS and completed the engine foundation. Now evolving it into a full game with 10 phases.

---

## Developer profile

- **Background:** JavaScript, TypeScript, Node.js
- **Rust level:** Intermediate — knows ownership, borrowing, structs, enums, traits, generics, closures, iterators, modules, serde. Does NOT know lifetimes, async/await, Arc/Mutex, trait objects yet.
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

### Current code architecture (pre-P1)

All state lives as `mut` variables in `main()`. Will be extracted to `GameState` in P1.

```
main.rs (game loop)
  │
  ├─ INPUT: SDL2 event pump → keyboard state + mouse clicks
  │   ├─ Mouse click → iso::screen_to_grid() → player.move_to() (A* pathfinding)
  │   └─ WASD held → player.try_move() (direct 1-tile move, cancels path)
  │
  ├─ UPDATE (fixed timestep, 60 ticks/sec):
  │   ├─ player.update() → advances path queue + lerps visual position
  │   ├─ camera.x/y = player.visual_x/y (smooth follow)
  │   └─ fov_map.compute() → 8-octant shadowcasting from player position
  │
  └─ RENDER:
      ├─ For each tile (row by row, back to front):
      │   ├─ iso::grid_to_screen() - camera offset → screen position
      │   ├─ Frustum cull (skip if off-screen)
      │   ├─ fov brightness → darken color
      │   └─ Draw filled diamond (scanline fill) + 3D side faces if wall
      ├─ Draw click target marker (yellow diamond)
      └─ Draw player (red body + head) on top of everything
```

### Key technical decisions
- **Scancode vs Keycode:** Using `Scancode` for movement (physical key position, layout-independent)
- **FOV uses distance falloff, not lerp:** Lerp-based FOV transition caused motion sickness. Distance falloff (100% brightness in inner 50% of radius, linear fade to edge) is instant but looks smooth because edges are already dim
- **Player draws on top of everything:** Known issue — proper depth interleaving caused flickering due to visual interpolation. Will fix in P3 with row-by-row entity sorting
- **Frustum culling:** Only draw tiles within screen bounds + 64px margin. Took 200x200 map from 4fps to 76fps

### Source files

| File | Lines | What it does |
|------|-------|-------------|
| `main.rs` | ~170 | SDL2 init, game loop (input→update→render), owns all mut state |
| `renderer.rs` | ~220 | `draw_world()`: tiles (filled diamonds + 3D walls), player sprite, click marker, FOV darkening. Takes canvas size dynamically for resize support |
| `fov.rs` | ~170 | `FovMap::compute()`: 8-octant recursive shadowcasting. `brightness: Vec<f64>` per tile with distance falloff. Walls (height > 0) block vision |
| `pathfinding.rs` | ~130 | `find_path()`: A* with Manhattan heuristic, 4-directional, `BinaryHeap` (reversed ordering for min-heap). Returns `Vec<Pos>` excluding start |
| `player.rs` | ~120 | `Player` struct: grid position (logical) + visual position (lerped f64). `try_move()` for WASD, `move_to()` for click (runs A*). `update()` advances path + lerps. Cooldown prevents rapid-click speed exploit |
| `tilemap.rs` | ~90 | `Tilemap` from JSON via serde. `TileKind` enum: Grass/Dirt/Water/Wall. Each has `top_color()`, `side_color()`, `height()`. Flat `Vec` with `row * cols + col` indexing |
| `iso.rs` | ~20 | `grid_to_screen(x,y)→(sx,sy)` and `screen_to_grid(sx,sy)→(x,y)`. Tile size: 64x32. Formula: `sx = (x-y)*32, sy = (x+y)*16` |
| `camera.rs` | ~12 | `Camera { x, y }` — viewport offset. Follows `player.visual_x/y` each tick |

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
| **P1** | GameState architecture — entity system, input/event pattern | **Next** |
| **P2** | Real sprites — PNG textures, AssetManager | Pending |
| **P3** | Multiple entities + depth fix — NPCs, interaction, text UI | Pending |
| **P4** | Audio + polish — music, SFX, animated tiles, particles | Pending |
| **P5** | D&D stats + inventory — character sheet, dice, items | Pending |
| **P6** | Turn-based combat + AI — initiative, actions, attacks | Pending |
| **P7** | Map transitions + save/load — connected world, interiors | Pending |
| **P8** | Dialogue + story — branching trees, quests, flags | Pending |
| **P9** | Multiplayer — tokio, client-server, up to 6 players co-op | Pending |
| **P10** | Map editor — visual tool for content creation | Pending |

### Dependencies per phase
P1: none | P2: sdl2 `image` | P3: sdl2 `ttf` | P4: sdl2 `mixer` | P5: `rand` | P6-P8: none | P9: `tokio`, `bincode` | P10: none

---

## Current Phase Detail — P1: GameState Architecture

**Goal:** Extract all state from main.rs into a `GameState` struct. Establish the input/event pattern that makes multiplayer possible later.

**What changes:**
- `src/game_state.rs` (new) — `GameState` struct owns: tilemap, `Vec<Entity>`, fov_map, click_target. Methods: `apply_input(&mut self, GameInput)`, `tick(&mut self) -> Vec<GameEvent>`
- `src/entity.rs` (new, replaces player.rs) — `Entity` struct with `id: u64`, `EntityKind` enum (Player/NPC/Enemy), grid + visual position, pathfinding state
- `src/input.rs` (new) — `GameInput` enum: `MoveDirection { entity_id, dx, dy }`, `MoveTo { entity_id, target_x, target_y }`
- `src/main.rs` — simplified to: SDL2 init → loop { poll events → translate to GameInput → apply → tick → render }
- `src/renderer.rs` — takes `&GameState` instead of individual pieces
- Add a second entity (NPC) standing on the map to prove multi-entity works

**Verification:** `cargo run` works identically to before. `main.rs` is ~50 lines. A second entity is visible.

---

## Reference

**The Rust Book:** https://doc.rust-lang.org/stable/book/

**Rust concepts not yet learned** (explain with JS/TS comparison before using):
Lifetimes (`'a`) → P2 | Arc/Mutex → P9 | async/await → P9 | trait objects (`dyn Trait`) → TBD | `#[serde(skip)]` → P7
