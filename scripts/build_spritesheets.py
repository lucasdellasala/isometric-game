"""
Build spritesheets from individual PNGs for faster game loading.
Run this after editing any sprite PNGs. The game loads these sheets
instead of individual files.

Output format per sheet:
  - 8 columns (directions: S, SW, W, NW, N, NE, E, SE)
  - N rows (frames)
  - Each cell: 256×512 px
  - src_rect for frame F, direction D: Rect(D*256, F*512, 256, 512)

Usage: python scripts/build_spritesheets.py
"""

import os
from pathlib import Path
from PIL import Image

FRAME_W = 256
FRAME_H = 512
DIRECTIONS = ["S", "SW", "W", "NW", "N", "NE", "E", "SE"]
NUM_FRAMES = 8

ROOT = Path(__file__).parent.parent
SPRITES = ROOT / "assets" / "sprites"
SHEETS = ROOT / "assets" / "spritesheets"


def build_sheet(name: str, frame_paths: list[list[str]], output_path: Path):
    """
    Build a spritesheet from a 2D list of frame paths.
    frame_paths[frame_idx][dir_idx] = path to PNG.
    Skips if all source PNGs are older than the output sheet.
    """
    # Check if rebuild is needed
    if output_path.exists():
        sheet_mtime = output_path.stat().st_mtime
        needs_rebuild = False
        for row in frame_paths:
            for p in row:
                if p and Path(p).exists() and Path(p).stat().st_mtime > sheet_mtime:
                    needs_rebuild = True
                    break
            if needs_rebuild:
                break
        if not needs_rebuild:
            return False

    num_frames = len(frame_paths)
    sheet = Image.new("RGBA", (FRAME_W * 8, FRAME_H * num_frames), (0, 0, 0, 0))

    missing = 0
    for frame_idx, row in enumerate(frame_paths):
        for dir_idx, path in enumerate(row):
            if path and Path(path).exists():
                img = Image.open(path)
                sheet.paste(img, (dir_idx * FRAME_W, frame_idx * FRAME_H))
            else:
                missing += 1

    output_path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(str(output_path))
    w, h = sheet.size
    print(f"  {name}: {w}x{h} ({missing} missing frames)")
    return True


def build_player():
    """Build player spritesheets: idle_static, idle_anim, walk."""
    idle_dir = SPRITES / "player" / "idle"
    walk_dir = SPRITES / "player" / "walk"

    # Static idle: 1 frame × 8 dirs
    paths = [[str(idle_dir / f"entity_player_{d}.png") for d in DIRECTIONS]]
    build_sheet("player_idle", paths, SHEETS / "player_idle.png")

    # Animated idle: 8 frames × 8 dirs
    paths = []
    for f in range(NUM_FRAMES):
        row = [str(idle_dir / f"entity_player_idle_{d}_{f}.png") for d in DIRECTIONS]
        paths.append(row)
    build_sheet("player_idle_anim", paths, SHEETS / "player_idle_anim.png")

    # Walk: 8 frames × 8 dirs
    paths = []
    for f in range(NUM_FRAMES):
        row = [str(walk_dir / f"entity_player_walk_{d}_{f}.png") for d in DIRECTIONS]
        paths.append(row)
    build_sheet("player_walk", paths, SHEETS / "player_walk.png")


def build_npcs():
    """Build NPC spritesheets for each variant."""
    npc_dir = SPRITES / "npc"
    variants = [d.name for d in npc_dir.iterdir() if d.is_dir() and not d.name.startswith("_")]

    for variant in sorted(variants):
        var_dir = npc_dir / variant
        prefix = f"entity_npc_{variant}"

        # Static idle
        paths = [[str(var_dir / f"{prefix}_{d}.png") for d in DIRECTIONS]]
        build_sheet(f"npc_{variant}_idle", paths, SHEETS / f"npc_{variant}_idle.png")

        # Animated idle
        idle_dir = var_dir / "idle"
        if idle_dir.exists():
            paths = []
            for f in range(NUM_FRAMES):
                row = [str(idle_dir / f"{prefix}_idle_{d}_{f}.png") for d in DIRECTIONS]
                paths.append(row)
            build_sheet(f"npc_{variant}_idle_anim", paths, SHEETS / f"npc_{variant}_idle_anim.png")

        # Walk
        walk_dir = var_dir / "walk"
        if walk_dir.exists():
            paths = []
            for f in range(NUM_FRAMES):
                row = [str(walk_dir / f"{prefix}_walk_{d}_{f}.png") for d in DIRECTIONS]
                paths.append(row)
            build_sheet(f"npc_{variant}_walk", paths, SHEETS / f"npc_{variant}_walk.png")


def build_enemies():
    """Build enemy spritesheets for each type."""
    enemy_dir = SPRITES / "enemy"
    types = [d.name for d in enemy_dir.iterdir() if d.is_dir() and not d.name.startswith("_")]

    for etype in sorted(types):
        type_dir = enemy_dir / etype
        prefix = f"entity_{etype}"

        # Static idle
        paths = [[str(type_dir / f"{prefix}_{d}.png") for d in DIRECTIONS]]
        build_sheet(f"enemy_{etype}_idle", paths, SHEETS / f"enemy_{etype}_idle.png")

        # Animated idle
        idle_dir = type_dir / "idle"
        if idle_dir.exists():
            paths = []
            for f in range(NUM_FRAMES):
                row = [str(idle_dir / f"{prefix}_idle_{d}_{f}.png") for d in DIRECTIONS]
                paths.append(row)
            build_sheet(f"enemy_{etype}_idle_anim", paths, SHEETS / f"enemy_{etype}_idle_anim.png")

        # Walk
        walk_dir = type_dir / "walk"
        if walk_dir.exists():
            paths = []
            for f in range(NUM_FRAMES):
                row = [str(walk_dir / f"{prefix}_walk_{d}_{f}.png") for d in DIRECTIONS]
                paths.append(row)
            build_sheet(f"enemy_{etype}_walk", paths, SHEETS / f"enemy_{etype}_walk.png")


if __name__ == "__main__":
    print("Building spritesheets...")
    build_player()
    build_npcs()
    build_enemies()
    print("Done!")
