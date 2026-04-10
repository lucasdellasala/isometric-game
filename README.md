# 🎮 Isometric RPG — D&D Co-op with DM System

![demo3 8mb](https://github.com/user-attachments/assets/c1e65cc9-43bd-4e02-b3e3-ffc87dd503e2)

A co-op RPG for up to 6 players, built from scratch in Rust with SDL2 — no game engine. Inspired by Baldur's Gate and Fallout 1: pre-rendered 3D graphics, isometric view, turn-based combat with D&D mechanics, and a system where any player can be the Dungeon Master.

This project is an experiment on two levels: learning Rust by building something ambitious, and exploring how far a solo dev can go using Claude Code as a real pair programmer.

## 🎯 The Vision

**You are the DM.** The game ships with official campaigns — complete stories with handcrafted maps, characters, quests and branching dialogue. But it also includes full tools for any player to build their own: an in-game map editor, NPC and enemy placement, and quest definition. Play the story, or tell your own.

**Enemies will think.** The planned combat AI uses an LLM to make tactical decisions in real time: it analyzes the terrain, the state of the players, and each creature's behavior profile from the Monster Manual. Every encounter will be unique.

**Real co-op.** Up to 6 simultaneous players, with the architecture designed for multiplayer from day one.

## 🔨 Current State

Early stage. The engine foundation exists — rendering, movement, pathfinding, FOV, basic NPCs — but there's no game yet. No combat, no stats, no story, no multiplayer. What you see in the GIF is a tech demo, not a game.

**Engine (done):**
- ✅ Isometric renderer with depth sorting and frustum culling
- ✅ 8-directional A* pathfinding with click-to-move
- ✅ FOV with 8-octant recursive shadowcasting
- ✅ Basic NPC entities with interaction and dialogue
- ✅ Pre-rendered graphics pipeline with Blender

**Game (not started yet):**
- 📋 D&D mechanics: stats, classes, inventory, dice
- 📋 Turn-based combat system
- 📋 Branching dialogue and quests
- 📋 Official campaigns and story
- 📋 In-game map editor and DM tools
- 📋 Multiplayer (up to 6 players)
- 🧪 LLM-driven enemy AI (experimental)

## 🚀 Run it

```bash
cargo run
```

## 📋 Requirements

- [Rust](https://rustup.rs/)
- [CMake](https://cmake.org/download/) (SDL2 compiles automatically via the `bundled` feature)

## 🛠️ Stack

- **Language:** Rust
- **Graphics:** SDL2 + pre-rendered sprites from Blender
- **Mechanics:** D&D SRD 5.2 (CC-BY-4.0)
- **No game engine.** The entire engine is custom code.

## 📖 Follow the Process

The full development is documented publicly. This project started as a learning experiment: a TypeScript dev learning Rust by doing it, with Claude Code as a real pair programmer — not just autocomplete, but for analyzing architecture, debugging, and making design decisions.

- 🐙 **Repo:** [github.com/lucasdellasala/isometric-game](https://github.com/lucasdellasala/isometric-game)
- 🎥 **YouTube:** [youtube.com/@vladyts](https://youtube.com/@vladyts)