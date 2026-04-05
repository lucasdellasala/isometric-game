# CLAUDE.md — Rust Learning + Isometric Game

## Project context

I'm learning Rust coming from JavaScript/TypeScript (Node.js). I understand programming — types, functions, data structures, async, etc. — and I've already completed the Rust fundamentals (Phase 1).

I'm now on **Phase 2 — building an isometric game** in the style of Fallout 1.

---

## My developer profile

- Languages I know: JavaScript, TypeScript, Node.js
- Concepts I already understand from JS: types, interfaces, generics (TS), async/await, closures, modules, basic data structures
- IDE: Zed (with built-in rust-analyzer)
- Environment: Windows 10, Rust 1.94.1, Claude Code as assistant
- I want to understand every line I write — not just make it work

---

## Phase 1 — Completed

Rust concepts I've learned and understand:

| Concept | Rust Book Chapter | Status |
|---------|-------------------|--------|
| Variables, mutability, `let` vs `const` | 3 | Done |
| Data types (`i32`, `u8`, `f64`, `bool`, `&str`, `String`) | 3 | Done |
| Functions, implicit return, macros vs functions | 3 | Done |
| Control flow (`if` as expression, `match`, `for`, `loop`, `while`) | 3 | Done |
| Ownership, move, Copy types | 4 | Done |
| Borrowing (`&`, `&mut`), borrowing rules | 4 | Done |
| Slices (`&str`, `&[T]`) | 4 | Done |
| Structs, methods, `impl`, associated functions | 5 | Done |
| Enums with data, exhaustive pattern matching | 6 | Done |
| `Option<T>` (`Some`/`None`) | 6 | Done |
| Collections: `Vec<T>`, `String`, `HashMap<K, V>` | 8 | Done |
| Error handling: `Result<T, E>`, `panic!`, `?` operator | 9 | Done |
| Traits, `impl Trait for Struct`, `derive`, trait bounds | 10 | Done |
| Generics `<T>` | 10 | Done |
| Closures (`\|x\| x + 1`), `move` | 13 | Done |
| Iterators (`.iter()`, `.map()`, `.filter()`, `.collect()`, etc.) | 13 | Done |
| Modules (`mod`, `use`, `pub`, folders with `mod.rs`, `crate::`) | 7 | Done |

### Rust concepts I haven't learned yet

These concepts will appear during Phase 2. **When any of them becomes necessary, explain it first with a small example before using it in the game, comparing with JS/TS.**

| Concept | When it will appear |
|---------|---------------------|
| **Lifetimes (`'a`)** | When a struct stores references instead of owned data |
| **Box, Rc, Arc** (smart pointers) | If we need dynamic polymorphism or shared data |
| **trait objects (`dyn Trait`)** | If we need different types in the same collection |
| **async/await** | Probably not in Phase 2, the game loop is synchronous |
| **unsafe** | If SDL2 requires it in some binding |
| **Lifetimes in structs** | When a struct needs to store a `&str` instead of `String` |
| **Custom error types** | When the game's error handling grows |
| **Closures as parameters (`Fn`, `FnMut`, `FnOnce`)** | When we pass callbacks to game systems |

---

## Interaction rules

### Explanations
- **Always compare with JS/TS** when introducing a new concept.
- **Explain the "why"** behind design decisions, not just the "how".
- If a concept has a chapter in The Rust Book, mention it.
- **If a milestone requires a new concept** (from the table above), explain it first with a small example before using it in the game.

### Code
- **Don't write code I can't explain.** Be ready to explain every line if I ask.
- **Prefer explicit code over idiomatic code** until I know both forms. Show the explicit one first, the idiomatic one second.
- **Always mentally compile before suggesting.** Don't suggest code the borrow checker will reject without warning.
- Use `cargo check` and `cargo clippy` as feedback tools.

### Pace
- One concept at a time.
- When something doesn't compile, **help me read the compiler error** instead of giving the solution directly.
- If the code uses a concept I haven't seen, **stop and explain before continuing**.

---

## Phase 2 — Isometric Game (IN PROGRESS)

### Concept
RPG/exploration with isometric view. Aesthetic similar to Fallout 1: pre-rendered 2D isometric graphics, tile-based, with fog of war and pathfinding.

### Tech stack
- **Language:** Rust
- **Graphics library:** SDL2 (`sdl2` crate with `bundled` feature)
- **No game engine.** All rendering, game loop, and systems are custom code.

### Milestones

| Milestone | Goal | Status |
|-----------|------|--------|
| **M1** | Window + game loop with fixed timestep | Done |
| **M2** | Isometric tile renderer (iso projection, grid, camera scroll) | Pending |
| **M3** | Sprites + correct depth sorting | Pending |
| **M4** | Tilemap from file (RON or JSON) | Pending |
| **M5** | Player entity + movement (click-to-move or WASD) | Pending |
| **M6** | Camera following player + large map | Pending |
| **M7** | A* pathfinding on isometric grid | Pending |
| **M8** | FOV / Shadowcasting (fog of war, line of sight) | Pending |

### Technical concepts to master in Phase 2
- Deterministic game loop: fixed timestep, update/render separation
- Isometric projection: screen <-> world coords conversion
- Depth sorting: z-ordering of tiles and entities in iso
- Sprite sheet blitting with SDL2
- A* on a grid with obstacles
- Shadowcasting for FOV (Bjorn Bergstrom's algorithm or similar)

---

## Reference

**The Rust Book:** https://doc.rust-lang.org/stable/book/

---

## JS -> Rust vocabulary (quick reference)

| JavaScript/TypeScript | Rust |
|-----------------------|------|
| `let x = 5` (mutable by default) | `let mut x = 5` (immutable by default) |
| `const x = 5` | `let x = 5` |
| `undefined` / `null` | `Option<T>` (`None` / `Some(value)`) |
| `throw new Error(...)` | `Err(...)` with `Result<T, E>` |
| `try/catch` | `match result { Ok(v) => ..., Err(e) => ... }` or `?` |
| `interface Foo { ... }` | `trait Foo { ... }` |
| `class Foo implements Bar` | `struct Foo; impl Bar for Foo { ... }` |
| `Array<T>` | `Vec<T>` |
| `Map<K, V>` | `HashMap<K, V>` |
| Closures `(x) => x + 1` | Closures `\|x\| x + 1` |
| ES Modules (`import/export`) | `mod`, `use`, `pub` |
| `typeof` / duck typing | Traits + compile-time generics |
| Garbage collector | Ownership + borrow checker (compile time) |
| `switch` (with fall-through) | `match` (exhaustive, no fall-through) |
| `?.` optional chaining | `if let Some(x) = ...` or `?` operator |
| `npm` / `package.json` | `cargo` / `Cargo.toml` |
| `node index.js` | `cargo run` |
| `eslint` | `cargo clippy` |

---

## Additional notes

- Always respond in Spanish.
- If there are multiple ways to do something, show the most explicit one first and the most idiomatic one second, explaining the difference.
- When the Rust compiler rejects something, help read the error message before giving the solution.
- The ultimate goal is a working game where I understand every line of code, not just that it works.
