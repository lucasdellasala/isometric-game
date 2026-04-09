---
name: render-npc-idle-spritesheet
description: Render an 8-direction isometric idle spritesheet (1024x256, 8 frames of 128x256) for an NPC body in Blender via the blender MCP. Use when the user asks to generate, render, or export an NPC/enemy idle spritesheet from a Blender body mesh. Produces a transparent PNG with an elliptical shadow under the feet, matching the project's NPC sprite format documented in CLAUDE.md.
---

# Render NPC idle spritesheet

Generates an 8-direction isometric idle spritesheet matching this project's convention:
- 1024×256 PNG, 8 frames of 128×256 laid out horizontally
- Transparent background
- Orthographic isometric camera, elevation 30°, rotating 45° per frame
- **Frame 0 = camera at SW of the body (compass 225°), then counterclockwise**: SW, S, SE, E, NE, N, NW, W
- **`ortho_scale = body_height`** (no padding multiplier — body fills the frame vertically)
- **Key light from upper-left of camera** (camera-space `(-X, +Y)` direction), shadowless, with boosted world background — body has almost no shadows on it
- Soft elliptical shadow under the feet (separate plane, not from light)
- Output saved to `assets/sprites/npc/<filename>.png`

## Inputs to confirm before running

Ask the user (or infer from context) the following, then plug them into the Blender script below:

1. **`BODY_NAME`** — name of the body MESH object in the Blender scene (e.g. `Body_Orc`, `Body_Human`).
2. **`EYE_NAMES`** — list of eye MESH object names that should render with the body (e.g. `["TF_EyeR_Orc", "TF_EyeL_Orc"]`). Pass `[]` if the body has no separate eye objects.
3. **`OUTPUT_NAME`** — filename without extension, following the project pattern `entity_npc_<race>[_<top>_<bottom>]` (e.g. `entity_npc_orc`, `entity_npc_human_bl_bn`).
4. **`EXTRA_VISIBLE`** — optional list of any extra objects that should also be visible during render (e.g. horns: `["TF_HornBase"]`). Default `[]`.

The script creates its own temporary `Sun` light per frame (positioned at upper-left of the camera) and hides every existing scene light during render, so the lighting is consistent across all 8 frames regardless of what lights exist in the scene.

## Backup existing sprites before overwriting

**Always back up any existing PNG with the same target filename before rendering**, so a bad render doesn't destroy a good one. The script must:

1. Create `assets/sprites/_backup/<timestamp>/` (where `<timestamp>` is `YYYYMMDD_HHMMSS`).
2. For every output PNG that already exists at the target path, **copy** (not move) it into the backup folder, preserving the relative subpath under `assets/sprites/` (e.g. `npc/entity_npc_orc.png` → `_backup/20260409_153012/npc/entity_npc_orc.png`).
3. Only after the backup copy succeeds, write the new PNG over the original location.

Use the same `<timestamp>` folder for all PNGs in a single render run (so a batch of 6 variants ends up in one backup folder, not six). Generate the timestamp once at the start.

Reference snippet to add at the top of any render script:

```python
import os, shutil, datetime
ASSETS_SPRITES = r"C:\Users\Urano\Documents\repositorios\rust\assets\sprites"
BACKUP_ROOT = os.path.join(ASSETS_SPRITES, "_backup",
                            datetime.datetime.now().strftime("%Y%m%d_%H%M%S"))

def backup_if_exists(out_path):
    if not os.path.exists(out_path):
        return
    rel = os.path.relpath(out_path, ASSETS_SPRITES)
    dst = os.path.join(BACKUP_ROOT, rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    shutil.copy2(out_path, dst)
```

Call `backup_if_exists(out_path)` immediately before every `sheet.save()` / `bpy.ops.render.render(write_still=True)` that targets a final asset path. Do **not** back up temp frame files inside `_tmp_*` folders — only the final composed sheets.

## Important rendering caveats

- **Disable Armature modifier on the body during render.** If the body has an `ARMATURE` modifier and the armature object is at world origin, the modifier will pull the deformed mesh back to (0,0,0), so a body whose `location` is at (5,0,0) won't actually render where you expect. The script temporarily sets `arm_mod.show_render = False` and restores it after.
- **Eye objects must be at the body's world position already** (not parented through an armature that re-snaps them). In this project we make per-variant eye copies offset by the body's `location`.
- **Save and restore everything.** Hide-render state of all objects, render engine, resolution, film transparency, filepath, image format, active camera. Delete the temp camera, shadow plane, shadow material, and temp frame PNGs after compositing the sheet.

## Blender script

Run this via `mcp__blender__execute_blender_code`. Edit the four variables at the top.

```python
import bpy, os, math
from mathutils import Vector

# ===== EDIT THESE =====
BODY_NAME     = "Body_Orc"
EYE_NAMES     = ["TF_EyeR_Orc", "TF_EyeL_Orc"]
EXTRA_VISIBLE = []  # e.g. ["TF_HornBase"] for tiefling
OUTPUT_NAME   = "entity_npc_orc"
# ======================

OUT_NPC = r"C:\Users\Urano\Documents\repositorios\rust\assets\sprites\npc"
TMP_DIR = os.path.join(OUT_NPC, "_tmp_" + OUTPUT_NAME)
os.makedirs(TMP_DIR, exist_ok=True)

body = bpy.data.objects[BODY_NAME]

# Disable armature so body renders at its actual location
arm_mod = next((m for m in body.modifiers if m.type == 'ARMATURE'), None)
prev_arm_render = arm_mod.show_render if arm_mod else None
if arm_mod:
    arm_mod.show_render = False

corners = [body.matrix_world @ Vector(c) for c in body.bound_box]
xs=[c.x for c in corners]; ys=[c.y for c in corners]; zs=[c.z for c in corners]
center = Vector(((min(xs)+max(xs))/2, (min(ys)+max(ys))/2, (min(zs)+max(zs))/2))
height = max(zs) - min(zs)
floor_z = min(zs)

# Hide everything; show only body+eyes+extras (NO scene lights — temp Sun is added)
saved_hide = {o.name: o.hide_render for o in bpy.data.objects}
for o in bpy.data.objects:
    o.hide_render = True
visible = [BODY_NAME] + EYE_NAMES + EXTRA_VISIBLE
for n in visible:
    if n in bpy.data.objects:
        bpy.data.objects[n].hide_render = False

# Shadow plane (independent from lighting)
bpy.ops.mesh.primitive_plane_add(size=1, location=(center.x, center.y, floor_z + 0.005))
shadow = bpy.context.active_object
shadow.name = "_RenderShadow_tmp"
shadow.scale = (0.55, 0.30, 1.0)
smat = bpy.data.materials.new("_RenderShadow_tmp_mat")
smat.use_nodes = True
sbsdf = smat.node_tree.nodes["Principled BSDF"]
sbsdf.inputs["Base Color"].default_value = (0, 0, 0, 1)
sbsdf.inputs["Roughness"].default_value = 1.0
sbsdf.inputs["Alpha"].default_value = 0.45
smat.blend_method = 'BLEND'
shadow.data.materials.append(smat)

# Camera — ortho_scale = body height (no padding multiplier)
cam_data = bpy.data.cameras.new("_RenderCam_tmp")
cam_data.type = 'ORTHO'
cam_data.ortho_scale = height
cam = bpy.data.objects.new("_RenderCam_tmp", cam_data)
bpy.context.scene.collection.objects.link(cam)

# Temp shadowless Sun (key light from upper-left of camera)
sun_data = bpy.data.lights.new("_RenderSun_tmp", type='SUN')
sun_data.energy = 5.0
sun_data.use_shadow = False
sun_obj = bpy.data.objects.new("_RenderSun_tmp", sun_data)
bpy.context.scene.collection.objects.link(sun_obj)

scene = bpy.context.scene

# Boost world background slightly so unlit sides aren't pitch black
prev_world_strength = None
if scene.world and scene.world.use_nodes:
    bg = scene.world.node_tree.nodes.get("Background")
    if bg:
        prev_world_strength = bg.inputs["Strength"].default_value
        bg.inputs["Strength"].default_value = 1.5

prev = {
    "camera": scene.camera,
    "engine": scene.render.engine,
    "resx": scene.render.resolution_x,
    "resy": scene.render.resolution_y,
    "film": scene.render.film_transparent,
    "filepath": scene.render.filepath,
    "color_mode": scene.render.image_settings.color_mode,
    "fmt": scene.render.image_settings.file_format,
}
scene.camera = cam
try:
    scene.render.engine = 'BLENDER_EEVEE_NEXT'
except Exception:
    scene.render.engine = 'BLENDER_EEVEE'
scene.render.resolution_x = 128
scene.render.resolution_y = 256
scene.render.film_transparent = True
scene.render.image_settings.file_format = 'PNG'
scene.render.image_settings.color_mode = 'RGBA'

# ===== ANGLE ORDER =====
# Frame 0 = camera at SW of body (compass 225°), then counterclockwise:
# SW(225) → S(180) → SE(135) → E(90) → NE(45) → N(0) → NW(315) → W(270)
# Compass: N=0, E=90, S=180, W=270.
# cam_x = cx + r * sin(compass_rad);  cam_y = cy + r * cos(compass_rad)
elev = math.radians(30.0)
distance = max(height * 2.5, 3.0)
r_h = distance * math.cos(elev)
z_off = distance * math.sin(elev)
look = Vector((center.x, center.y, center.z))

start_compass_deg = 225.0
step_deg = -45.0  # counterclockwise

frames = []
for i in range(8):
    compass = math.radians(start_compass_deg + i * step_deg)
    cam.location = (
        center.x + r_h * math.sin(compass),
        center.y + r_h * math.cos(compass),
        center.z + z_off,
    )
    direction = look - cam.location
    cam.rotation_euler = direction.to_track_quat('-Z', 'Y').to_euler()
    bpy.context.view_layer.update()

    # Light from upper-left of camera, recomputed each frame so the relative
    # lighting on the body is identical in every direction.
    cam_mat3 = cam.matrix_world.to_3x3()
    cam_right = cam_mat3.col[0]
    cam_up    = cam_mat3.col[1]
    light_dir_from_orc = (cam_up - cam_right).normalized()
    sun_obj.location = look + light_dir_from_orc * 6.0
    sun_to_orc = look - sun_obj.location
    sun_obj.rotation_euler = sun_to_orc.to_track_quat('-Z', 'Y').to_euler()

    fp = os.path.join(TMP_DIR, f"frame_{i}.png")
    scene.render.filepath = fp
    bpy.ops.render.render(write_still=True)
    frames.append(fp)

# Compose 1024x256
sheet_w, sheet_h = 1024, 256
sheet = bpy.data.images.new("_sheet_tmp", sheet_w, sheet_h, alpha=True)
buf = [0.0] * (sheet_w * sheet_h * 4)
for idx, fp in enumerate(frames):
    img = bpy.data.images.load(fp)
    px = list(img.pixels)
    fw, fh = img.size
    x_off = idx * 128
    for y in range(fh):
        s = (y * fw) * 4
        d = (y * sheet_w + x_off) * 4
        buf[d:d + fw*4] = px[s:s + fw*4]
    bpy.data.images.remove(img)
sheet.pixels = buf
out_path = os.path.join(OUT_NPC, OUTPUT_NAME + ".png")
sheet.filepath_raw = out_path
sheet.file_format = 'PNG'
sheet.save()
print("Saved:", out_path)

# Cleanup
bpy.data.images.remove(sheet)
bpy.data.objects.remove(shadow, do_unlink=True)
bpy.data.materials.remove(smat)
bpy.data.objects.remove(cam, do_unlink=True)
bpy.data.cameras.remove(cam_data)
bpy.data.objects.remove(sun_obj, do_unlink=True)
bpy.data.lights.remove(sun_data)

if prev_world_strength is not None and scene.world and scene.world.use_nodes:
    bg = scene.world.node_tree.nodes.get("Background")
    if bg:
        bg.inputs["Strength"].default_value = prev_world_strength

scene.camera = prev["camera"]
scene.render.engine = prev["engine"]
scene.render.resolution_x = prev["resx"]
scene.render.resolution_y = prev["resy"]
scene.render.film_transparent = prev["film"]
scene.render.filepath = prev["filepath"]
scene.render.image_settings.color_mode = prev["color_mode"]
scene.render.image_settings.file_format = prev["fmt"]

for name, hr in saved_hide.items():
    if name in bpy.data.objects:
        bpy.data.objects[name].hide_render = hr
if arm_mod is not None:
    arm_mod.show_render = prev_arm_render

for fp in frames:
    try: os.remove(fp)
    except Exception: pass
try: os.rmdir(TMP_DIR)
except Exception: pass
print("DONE")
```

## After running

1. Tell the user the output path.
2. Mention the four things that are commonly tweaked and offer to re-run with adjustments:
   - Frame order (counterclockwise from front; engine may expect S/SE/E/NE/N/NW/W/SW)
   - Framing (`ortho_scale = height * 1.15` — increase for more padding)
   - Shadow size (`shadow.scale`) and opacity (`Alpha` on shadow material)
   - Lighting — uses scene `Sun` only by default
