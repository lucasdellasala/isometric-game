# Blender character pipeline — variantes + spritesheet idle

Documento de referencia para reconstruir el sistema de personajes (humano, tiefling, orco) en `assets_dev/sculp_base.blend` y renderizar spritesheets idle isométricos para el juego.

> **Importante**: Blender crasheó después de hacer todo este trabajo y no estaba guardado. Este documento permite reconstruir desde el estado original (un solo `TF_Body_Sculpt`, sin variantes) hasta el estado de trabajo (3 variantes con materiales y ojos independientes + render rig montado). **Guardá el .blend (`Ctrl+S`) seguido cada vez que termines un paso.**

---

## 1. Estado de partida (post-crash)

La escena tiene:
- `TF_Body_Sculpt` — cuerpo único (multires + armature, 10582 vértices base)
- `TF_Body` — versión vieja oculta
- `TF_EyeR`, `TF_EyeL` — ojos en `(±0.03, -0.09, 1.58)` pero con un `matrix_parent_inverse` con rotación 45° en Z metida adentro: aparecen flotando fuera de la cara
- `TF_HornBase` — cuernos sin posicionar, en `(0, -11.98, 7)`, con scale 100 y parent al `Armature`
- `TF_Shadow` — sombra del piso del tiefling
- `Armature` — esqueleto del tiefling
- `Camera`, `ISO_Cam`, `Sun`
- 7 materiales (incluye `Body_Skin`, `Body_Top`, `Body_Bottom`, `TF_Eye_Iris`, `TF_Eye_White`, `TF_Horn_Black`)

---

## 2. Convenciones del proyecto

### Compass / orden de frames del spritesheet

El spritesheet de NPC idle es **1024×256** con **8 frames de 128×256** lado a lado. El orden, de izquierda a derecha:

| Frame | Dir | Significado |
|-------|-----|-------------|
| 1 | **SW** | El frente del modelo apunta SW (esquina inferior-izquierda en pantalla iso) |
| 2 | S  | Frente apunta hacia abajo de la pantalla |
| 3 | SE | |
| 4 | E  | |
| 5 | NE | |
| 6 | **N**  | El modelo da la espalda a la cámara |
| 7 | NW | |
| 8 | W  | |

El cuerpo del orco/tiefling tiene su nariz natural en `-Y` (del mesh). Con la cámara isométrica del proyecto en el cuadrante `+X, -Y, +Z` mirando al origen, **la pose natural (rotación Z = 0°) es la pose `SW`**. Cada frame siguiente rota el cuerpo 45° CCW alrededor de Z.

**Tabla de rotaciones del pivote** (todos los valores en grados):

| Frame | Dir | `pivot.rotation_euler.z` |
|-------|-----|--------------------------|
| 1 | SW | 0   |
| 2 | S  | 45  |
| 3 | SE | 90  |
| 4 | E  | 135 |
| 5 | NE | 180 |
| 6 | N  | 225 |
| 7 | NW | 270 |
| 8 | W  | 315 |

### Cámara isométrica

- **Tipo:** Ortográfica
- **`ortho_scale = altura del cuerpo`** (sin multiplicador de padding — el cuerpo llena el frame verticalmente)
- **Posición:** `(d, -d, h)` con `d ≈ height * 2.5 * cos(30°)` y `h = d * tan(30°)` (elevación 30°)
- **Orientación:** apuntando al centro del cuerpo
- **Resolución de render:** 128×256, film transparent, PNG RGBA

### Iluminación

Tres palancas. Valores que funcionaron bien:

| Parámetro | Valor | Dónde |
|-----------|-------|-------|
| Sun **Strength (Energy)** | **3.5** | `OrcRender_KeySun` → Object Data Properties |
| Sun **shadows** | **OFF** (`use_shadow = False`) | mismo lugar, "Cast Shadow" |
| **World Background Strength** | **3.0** | Properties → World → Surface → Background → Strength |
| Posición del Sun | upper-left de la cámara, en *camera space* `(-X, +Y)` | `OrcRender_KeySun.location` |

**Reglas para tunear:**
- **Cero sombras propias / look plano cartoon:** Sun 1.5–2.0, Background 5–6.
- **Más volumen / sombras suaves marcadas:** Sun 5–6, Background 1.5.
- **Tinte ambiental:** cambiá el `Color` del Background (azulado = frío, anaranjado = cálido).
- **Mover dirección de la luz:** mové `OrcRender_KeySun` con `G`. Si pierde el ángulo, ajustá `rotation_euler` a mano o agregale un `Track To` constraint apuntando al cuerpo.

### Caveat crítico del render: Armature modifier

`Body_Orc`, `Body_Human` y `TF_Body_Sculpt` tienen un modifier `ARMATURE` apuntando al objeto `Armature` (en el origen). **Si el modifier está activo, el cuerpo se "succiona" de vuelta a (0,0,0)** independientemente del `object.location`. Esto rompe el rendering de las variantes que están en X≠0.

**Solución:** antes de renderizar, desactivá `show_viewport = False` y `show_render = False` en el modifier Armature de la variante a renderizar. Reactivá después si querés posar con el armature.

---

## 3. Reconstrucción desde el estado actual

Cada paso es un bloque Python para pegar en la consola scripting de Blender (o ejecutar vía MCP). **Guardá después de cada paso.**

### Paso 1 — Arreglar la posición de los ojos del tiefling

Los ojos tienen un `matrix_parent_inverse` con rotación de 45° en Z que los manda fuera de la cara. Hay que limpiarlo y resetear sus transforms.

```python
import bpy
from mathutils import Matrix, Euler
for n, x in (("TF_EyeR", 0.033), ("TF_EyeL", -0.033)):
    o = bpy.data.objects[n]
    o.matrix_parent_inverse = Matrix.Identity(4)
    o.location = (x, -0.087, 1.58)
    o.rotation_euler = Euler((1.5707963, 0.0, 0.0), 'XYZ')
bpy.context.view_layer.update()
```

Verificación: `TF_EyeR` queda con bbox X∈[+0.021,+0.045], `TF_EyeL` con X∈[-0.045,-0.021], ambos a Y≈-0.075..-0.099, Z≈1.568..1.592.

### Paso 2 — Reposicionar los cuernos en la cabeza del tiefling

`TF_HornBase` está flotando lejísimos (centro mundial cerca de `(0, 1.69, 0.04)`). Hay que llevarlo a la corona del cráneo manteniendo el parent del armature.

```python
import bpy
from mathutils import Vector, Matrix
h = bpy.data.objects["TF_HornBase"]
hcorners = [h.matrix_world @ Vector(c) for c in h.bound_box]
cur_center = sum(hcorners, Vector()) / 8.0
target_center = Vector((0.0, -0.02, 1.72))   # encima del cráneo
delta = target_center - cur_center
h.matrix_world = Matrix.Translation(delta) @ h.matrix_world
```

Verificación: bbox del horn queda ~ x∈[-0.11,+0.11], y∈[-0.11,+0.07], z∈[1.65,1.79].

### Paso 3 — Pintar la piel del tiefling carmín y los cuernos rojo hueso

```python
import bpy
# Skin carmín oscuro
mat = bpy.data.materials["Body_Skin"]
mat.use_nodes = True
bsdf = mat.node_tree.nodes["Principled BSDF"]
carmine = (0.25, 0.02, 0.04, 1.0)
bsdf.inputs["Base Color"].default_value = carmine
mat.diffuse_color = carmine

# Cuernos rojo hueso (mate)
mat = bpy.data.materials["TF_Horn_Black"]
mat.use_nodes = True
bsdf = mat.node_tree.nodes["Principled BSDF"]
bone_red = (0.16, 0.07, 0.07, 1.0)
bsdf.inputs["Base Color"].default_value = bone_red
bsdf.inputs["Roughness"].default_value = 0.7
mat.diffuse_color = bone_red
mat.name = "TF_Horn_BoneRed"
```

### Paso 4 — Crear las variantes Human y Orc

Copia de `TF_Body_Sculpt` con **mesh data independiente** (cada variante tiene su propio sculpt) y **materiales independientes** (cambiar la piel del orco no afecta al humano ni al tiefling). Las variantes se posicionan offset en X para que se vean lado a lado en la escena de trabajo.

```python
import bpy
src = bpy.data.objects["TF_Body_Sculpt"]
variants = [("Body_Human", (2.5, 0, 0), "Human"),
            ("Body_Orc",   (5.0, 0, 0), "Orc")]
for name, loc, suffix in variants:
    new_obj = src.copy()
    new_obj.data = src.data.copy()         # mesh data independiente (incluye multires)
    new_obj.data.name = name + "_Mesh"
    new_obj.name = name
    new_obj.location = loc
    new_obj.parent = None
    new_obj.matrix_parent_inverse.identity()
    bpy.context.scene.collection.objects.link(new_obj)
    # Materiales independientes
    for i, slot in enumerate(new_obj.data.materials):
        if slot is None: continue
        new_mat = slot.copy()
        new_mat.name = f"{slot.name}_{suffix}"
        new_obj.data.materials[i] = new_mat
```

`TF_Body_Sculpt` queda como definitivo del **tiefling** con sus materiales originales (`Body_Skin`, `Body_Top`, `Body_Bottom`).

### Paso 5 — Crear ojos por variante

Cada variante necesita su par de ojos con materiales independientes (para que el iris pueda ser de un color distinto sin afectar a las demás).

```python
import bpy
from mathutils import Matrix
variants = {
    "Human": {"body": "Body_Human", "iris": (0.04, 0.12, 0.35, 1.0)},   # azul
    "Orc":   {"body": "Body_Orc",   "iris": (0.55, 0.28, 0.04, 1.0)},   # ámbar
}
for vname, info in variants.items():
    body = bpy.data.objects[info["body"]]
    body_offset = body.location
    iris_mat = bpy.data.materials["TF_Eye_Iris"].copy()
    iris_mat.name = f"Eye_Iris_{vname}"
    iris_mat.use_nodes = True
    bsdf = iris_mat.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = info["iris"]
    iris_mat.diffuse_color = info["iris"]
    white_mat = bpy.data.materials["TF_Eye_White"].copy()
    white_mat.name = f"Eye_White_{vname}"
    for src_name in ("TF_EyeR", "TF_EyeL"):
        src = bpy.data.objects[src_name]
        new_name = f"{src_name}_{vname}"
        new_obj = src.copy()
        new_obj.data = src.data.copy()
        new_obj.data.name = new_name + "_Mesh"
        new_obj.name = new_name
        new_obj.parent = None
        new_obj.matrix_parent_inverse.identity()
        new_obj.matrix_world = Matrix.Translation(body_offset) @ src.matrix_world
        for i, slot in enumerate(new_obj.data.materials):
            if slot is None: continue
            if slot.name.startswith("TF_Eye_Iris"):
                new_obj.data.materials[i] = iris_mat
            elif slot.name.startswith("TF_Eye_White"):
                new_obj.data.materials[i] = white_mat
```

### Paso 6 — Paletas de color por variante

```python
import bpy
presets = {
    "Human": {
        "Body_Skin_Human":   (0.55, 0.30, 0.20, 1.0),  # tan cálido
        "Body_Top_Human":    (0.10, 0.20, 0.40, 1.0),  # tunica azul
        "Body_Bottom_Human": (0.08, 0.05, 0.03, 1.0),  # marrón oscuro
    },
    "Orc": {
        "Body_Skin_Orc":   (0.10, 0.22, 0.07, 1.0),    # verde apagado oscuro
        "Body_Top_Orc":    (0.18, 0.09, 0.04, 1.0),    # cuero marrón
        "Body_Bottom_Orc": (0.04, 0.04, 0.04, 1.0),    # casi negro
    },
}
for variant, mats in presets.items():
    for mat_name, color in mats.items():
        mat = bpy.data.materials[mat_name]
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes["Principled BSDF"]
        bsdf.inputs["Base Color"].default_value = color
        if "Skin" in mat_name:
            bsdf.inputs["Roughness"].default_value = 0.6
        mat.diffuse_color = color
```

### Paso 7 — Sculpt manual del orco

Esto es manual, no hay script. Las copias arrancan idénticas al `TF_Body_Sculpt` (con sus orejas tiefling puntiagudas y rasgos). Hay que sculptarlas para diferenciarlas:

- **Body_Orc:** seleccionar → Sculpt Mode → ensanchar mandíbula, agregar colmillos, redondear orejas (o hacerlas más anchas), ajustar cejas. La mesh data ya es independiente (`Body_Orc_Mesh`), así que sculptar acá NO afecta al tiefling.
- **Body_Human:** suavizar puntas de orejas, suavizar rasgos faciales para algo más humano.

> Tip: el modificador Multires está activo con `sculpt_levels = 3` y `render_levels = 3`. Sculptá en niveles altos para detalle, en niveles bajos para forma general.

---

## 4. Render rig persistente

Setup de cámara + pivote + luz que se queda en la escena para poder previsualizar y renderizar cada variante.

### Crear el rig (ejemplo: Body_Orc)

```python
import bpy, math
from mathutils import Vector

BODY_NAME = "Body_Orc"
EYE_NAMES = ["TF_EyeR_Orc", "TF_EyeL_Orc"]
PREFIX    = "OrcRender"   # cambiar por HumanRender / TieflingRender según el caso

body  = bpy.data.objects[BODY_NAME]
eye_r = bpy.data.objects[EYE_NAMES[0]]
eye_l = bpy.data.objects[EYE_NAMES[1]]

# 1) Desactivar armature modifier (viewport + render)
arm_mod = next((m for m in body.modifiers if m.type == 'ARMATURE'), None)
if arm_mod:
    arm_mod.show_viewport = False
    arm_mod.show_render = False

# 2) Bbox del cuerpo
corners = [body.matrix_world @ Vector(c) for c in body.bound_box]
xs=[c.x for c in corners]; ys=[c.y for c in corners]; zs=[c.z for c in corners]
center = Vector(((min(xs)+max(xs))/2, (min(ys)+max(ys))/2, (min(zs)+max(zs))/2))
height = max(zs) - min(zs)
floor_z = min(zs)

# 3) Pivot empty (en la base del cuerpo)
pivot = bpy.data.objects.new(PREFIX + "_Pivot", None)
pivot.empty_display_type = 'ARROWS'
pivot.empty_display_size = 0.4
bpy.context.scene.collection.objects.link(pivot)
pivot.location = (center.x, center.y, floor_z)
pivot.rotation_euler = (0, 0, 0)   # frame 1 (SW)
bpy.context.view_layer.update()

# 4) Parentar body + ojos al pivote (preservando world)
for ob in (body, eye_r, eye_l):
    mw = ob.matrix_world.copy()
    ob.parent = pivot
    ob.matrix_parent_inverse = pivot.matrix_world.inverted() @ mw @ ob.matrix_local.inverted()
    ob.matrix_world = mw

# 5) Cámara orto isométrica fija
cam_data = bpy.data.cameras.new(PREFIX + "_Cam")
cam_data.type = 'ORTHO'
cam_data.ortho_scale = height
cam = bpy.data.objects.new(PREFIX + "_Cam", cam_data)
bpy.context.scene.collection.objects.link(cam)
elev = math.radians(30.0)
distance = max(height * 2.5, 3.0)
horizontal = distance * math.cos(elev)
cam.location = (
    center.x + horizontal * math.sin(math.radians(45)),
    center.y - horizontal * math.cos(math.radians(45)),
    center.z + distance * math.sin(elev),
)
look = Vector((center.x, center.y, center.z))
cam.rotation_euler = (look - cam.location).to_track_quat('-Z', 'Y').to_euler()
bpy.context.scene.camera = cam

# 6) Render settings
scene = bpy.context.scene
scene.render.resolution_x = 128
scene.render.resolution_y = 256
scene.render.film_transparent = True

# 7) Sun (key light) sin sombras desde upper-left de la cámara
sun_data = bpy.data.lights.new(PREFIX + "_KeySun", type='SUN')
sun_data.energy = 3.5
sun_data.use_shadow = False
sun_obj = bpy.data.objects.new(PREFIX + "_KeySun", sun_data)
bpy.context.scene.collection.objects.link(sun_obj)
cam_mat3 = cam.matrix_world.to_3x3()
light_dir = (cam_mat3.col[1] - cam_mat3.col[0]).normalized()  # cam_up - cam_right
sun_obj.location = look + light_dir * 6.0
sun_obj.rotation_euler = (look - sun_obj.location).to_track_quat('-Z', 'Y').to_euler()

# 8) World background (relleno ambiental)
if scene.world and scene.world.use_nodes:
    bg = scene.world.node_tree.nodes.get("Background")
    if bg:
        bg.inputs["Strength"].default_value = 3.0
```

### Previsualizar otros frames

Seleccionar el `OrcRender_Pivot` en el outliner y cambiar `rotation_euler.z` (en grados, vía N-panel). Tabla en sección 2.

### Renderizar el spritesheet completo (1024×256)

Cuando el rig esté armado y la iluminación tuneada, este script rota el pivote y compone los 8 frames:

```python
import bpy, os, math
PREFIX = "OrcRender"
OUTPUT_NAME = "entity_npc_orc"   # sin extensión
OUT_DIR = r"C:\Users\Urano\Documents\repositorios\rust\assets\sprites\enemy"

pivot = bpy.data.objects[PREFIX + "_Pivot"]
cam   = bpy.data.objects[PREFIX + "_Cam"]
scene = bpy.context.scene
scene.camera = cam
scene.render.image_settings.file_format = 'PNG'
scene.render.image_settings.color_mode = 'RGBA'
scene.render.resolution_x = 128
scene.render.resolution_y = 256
scene.render.film_transparent = True

tmp = os.path.join(OUT_DIR, "_tmp_" + OUTPUT_NAME)
os.makedirs(tmp, exist_ok=True)
frames = []
for i in range(8):
    pivot.rotation_euler.z = math.radians(i * 45)
    bpy.context.view_layer.update()
    fp = os.path.join(tmp, f"frame_{i}.png")
    scene.render.filepath = fp
    bpy.ops.render.render(write_still=True)
    frames.append(fp)

# Componer 1024x256
sheet = bpy.data.images.new("_sheet_tmp", 1024, 256, alpha=True)
buf = [0.0] * (1024 * 256 * 4)
for idx, fp in enumerate(frames):
    img = bpy.data.images.load(fp)
    px = list(img.pixels)
    for y in range(256):
        s = (y * 128) * 4
        d = (y * 1024 + idx * 128) * 4
        buf[d:d + 128*4] = px[s:s + 128*4]
    bpy.data.images.remove(img)
sheet.pixels = buf
out_path = os.path.join(OUT_DIR, OUTPUT_NAME + ".png")
sheet.filepath_raw = out_path
sheet.file_format = 'PNG'
sheet.save()
bpy.data.images.remove(sheet)

# Volver el pivote a SW
pivot.rotation_euler.z = 0
for fp in frames:
    try: os.remove(fp)
    except Exception: pass
try: os.rmdir(tmp)
except Exception: pass
print("Sheet saved:", out_path)
```

> Para el humano y el tiefling, repetir la sección "Crear el rig" cambiando `BODY_NAME`, `EYE_NAMES` y `PREFIX`. El tiefling además necesita `TF_HornBase` parented al mismo pivote (agregalo a la lista de objetos a re-parentar).

### Output paths convención

Todos los sprites son 256×512 px (2× la resolución original de 128×256 para soportar zoom sin pixelarse). Cada raza/variante tiene su propia subcarpeta con 8 PNGs individuales (uno por dirección cardinal: S, SW, W, NW, N, NE, E, SE).

```
assets/sprites/
├── player/
│   ├── idle/
│   │   ├── entity_player_S.png ... entity_player_SE.png     (8 PNGs, 256×512)
│   ├── walk/
│   │   ├── entity_player_walk_S_0.png ... _SE_7.png          (64 PNGs, 256×512)
│   └── entity_shadow.png                                      (256×128)
├── enemy/
│   └── orc/
│       ├── entity_orc_S.png ... entity_orc_SE.png            (8 idle, 256×512)
│       └── walk/
│           ├── entity_orc_walk_S_0.png ... _SE_7.png          (64 PNGs, 256×512)
└── npc/
    ├── african_cr_bk/
    │   ├── entity_npc_african_cr_bk_S.png ... _SE.png         (8 PNGs, 256×512)
    ├── african_gn_cr/
    ├── caucasian_gn_bn/
    ├── caucasian_yl_bk/
    ├── latino_bk_bn/
    └── latino_yl_bk/
```

**Naming:**
- Player: `entity_player_{cardinal}.png` (idle), `entity_player_walk_{cardinal}_{frame}.png` (walk)
- Enemy: `entity_orc_{cardinal}.png` (idle), `entity_orc_walk_{cardinal}_{frame}.png` (walk)
- NPC: `entity_npc_{ethnicity}_{top}_{bottom}_{cardinal}.png` (idle), dentro de subcarpeta `{ethnicity}_{top}_{bottom}/`

**ortho_scale por raza:**
- Tiefling/Humano (idle): `1.69 * 1.05 / 0.9 ≈ 1.972` (10% más chico que orco)
- Orco (idle): `1.69 * 1.05 ≈ 1.7745`
- Orco (walk): `2.90` (bbox animado más grande por la zancada)
- Tiefling (walk): `1.69 * 1.05 / 0.9 ≈ 1.972` (pendiente calcular bbox animado si se corta)

**Shadow:** se renderiza como sprite independiente (`entity_shadow.png`, 256×128). El código Rust lo dibuja debajo del personaje, encima del tile. NO se incluye en los sprites de personaje.

---

## 5. Cómo evitar el próximo crash

- **Guardá (`Ctrl+S`) seguido**, sobre todo después de:
  - Crear o duplicar objetos
  - Aplicar modificadores
  - Cambiar materiales o sculpts
- Activá Auto-Save: `Edit > Preferences > Save & Load > Auto Save` cada 2–5 minutos.
- Antes de scripts pesados (composición de imágenes pixel-por-pixel, render de muchos frames), guardá.

---

## 6. Sistema de "traje" — armature compartido + meshes intercambiables

### Concepto

Hay **un único `Armature`** en la escena que contiene el esqueleto y todas las animaciones (idle, walk, etc.). Hay **N body meshes** (uno por raza) que comparten:

1. **Misma topología** (mismo número de vértices en el mismo orden) — por copia de un body master
2. **Vertex groups con los mismos nombres** que los bones del armature (`mixamorig:Hips`, `mixamorig:Spine`, etc.)
3. **Modificador Armature apuntando al mismo objeto Armature**, **arriba** del Multires

Cuando el armature se anima, todos los bodies se deforman idénticamente. Para renderizar una raza particular, ocultás las otras. Es como ponerle "un traje distinto" a la misma animación.

### Jerarquía actual de la escena

```
Camera, ISO_Cam, Sun           (scene infrastructure)
Armature                       (Mixamo, 65 bones, action "Armature|mixamo.com|Layer0" frames 1..32)
Tiefling                       (Empty parent at world origin)
   ├─ TF_Body_Sculpt           — body, modifiers [ARMATURE → MULTIRES]
   ├─ TF_EyeR, TF_EyeL         — eye meshes, Child Of constraint → mixamorig:Head
   ├─ TF_HornBase              — horn mesh, Child Of constraint → mixamorig:Head
   ├─ TF_Shadow                — fake shadow plane
   └─ TF_Body                  — legacy lowpoly (hidden, optional to delete)
Human                          (Empty parent)
   ├─ Body_Human               — body, modifiers [ARMATURE → MULTIRES]
   └─ TF_EyeR_Human, TF_EyeL_Human  — Child Of constraint → mixamorig:Head
Orc                            (Empty parent)
   ├─ Body_Orc                 — body, modifiers [ARMATURE → MULTIRES]
   └─ TF_EyeR_Orc, TF_EyeL_Orc — Child Of constraint → mixamorig:Head
```

`Tiefling`, `Human`, `Orc` son los únicos "personajes" raíz. Todo lo del cuerpo (mesh + ojos + cuernos + sombra) cuelga de su Empty correspondiente. El `Armature` queda en root level porque es **infraestructura compartida**, no pertenece a ningún personaje en particular.

### Naming reference

| Tipo | Patrón | Ejemplos |
|------|--------|----------|
| Personaje raíz | `<Race>` (Empty) | `Tiefling`, `Human`, `Orc` |
| Body mesh | `Body_<Race>` o legacy `TF_Body_Sculpt` | `Body_Human`, `Body_Orc`, `TF_Body_Sculpt` |
| Mesh data | `<ObjectName>_Mesh` | `Body_Human_Mesh`, `Body_Orc_Mesh` |
| Eye object | `TF_Eye{R\|L}_<Race>` o legacy `TF_Eye{R\|L}` | `TF_EyeR_Human`, `TF_EyeL_Orc` |
| Horns (solo tiefling) | `TF_HornBase` | |
| Skin material | `Body_Skin_<Race>` o legacy `Body_Skin` | `Body_Skin_Human`, `Body_Skin_Orc` |
| Top material | `Body_Top_<Race>` | `Body_Top_Human`, `Body_Top_Orc` |
| Bottom material | `Body_Bottom_<Race>` | `Body_Bottom_Human`, `Body_Bottom_Orc` |
| Iris material | `Eye_Iris_<Race>` o legacy `TF_Eye_Iris` | `Eye_Iris_Human`, `Eye_Iris_Orc` |
| Eye white material | `Eye_White_<Race>` o legacy `TF_Eye_White` | `Eye_White_Human`, `Eye_White_Orc` |
| Horn material | `TF_Horn_BoneRed` (renombrado del original `TF_Horn_Black`) | |

> El **tiefling** quedó con los nombres legacy `TF_*` porque era el master que ya existía antes de armar el sistema. Las razas nuevas usan el patrón `<Tipo>_<Race>`.

### Cómo importar el FBX de Mixamo correctamente

```python
import bpy
bpy.ops.import_scene.fbx(
    filepath=r"C:\Users\Urano\Documents\repositorios\rust\assets_dev\Walking.fbx",
    automatic_bone_orientation=True,   # CRÍTICO: corrige el problema Y-up de Mixamo
    use_anim=True,
)
```

Sin `automatic_bone_orientation=True` los bones quedan acostados horizontalmente (espina por +Y mundial en lugar de +Z) y todo se rompe. Con la flag activa, los bones llegan en orientación correcta (Hips z≈0.9, Head z≈1.5, Feet z≈0).

Después del import:
- Aparece un nuevo objeto `Armature` (escala 0.01, 65 bones nombrados `mixamorig:*`)
- Aparece un mesh genérico de Mixamo (ej. `TF_Body.001`) — **borrarlo**, no lo necesitamos porque usamos nuestros bodies
- Aparece la action `Armature|mixamo.com|Layer0` con range [1, 32]

### Cómo bindear un body al armature

```python
import bpy

body = bpy.data.objects["Body_<Race>"]
arm = bpy.data.objects["Armature"]

# Add Armature modifier and move it ABOVE Multires
arm_mod = body.modifiers.new(name="Armature", type='ARMATURE')
arm_mod.object = arm
arm_mod.use_vertex_groups = True
arm_idx = list(body.modifiers).index(arm_mod)
if arm_idx != 0:
    body.modifiers.move(arm_idx, 0)
```

Resultado: stack `[ARMATURE, MULTIRES]`. **El orden importa**: Armature primero deforma el lowpoly base, después Multires subdivide y agrega el sculpt detail. Invertido (`MULTIRES, ARMATURE`) rompe el sculpt durante la animación.

Como el body comparte los mismos vertex groups `mixamorig:*` que los bones del armature, el binding funciona automáticamente sin re-skinning ni weight painting.

### Cómo attachear ojos / cuernos al head bone (Child Of constraint)

```python
import bpy

arm = bpy.data.objects["Armature"]
HEAD_BONE = "mixamorig:Head"

# CRÍTICO: setear inverse_matrix en REST pose para que el binding sea consistente
prev_pose = arm.data.pose_position
arm.data.pose_position = 'REST'
bpy.context.view_layer.update()

parent_mat = arm.matrix_world @ arm.pose.bones[HEAD_BONE].matrix
parent_mat_inv = parent_mat.inverted()

for name in ("TF_EyeR_<Race>", "TF_EyeL_<Race>"):  # + horns si la raza los tiene
    o = bpy.data.objects[name]

    # Quitar Child Of viejos si los hay
    for c in list(o.constraints):
        if c.type == 'CHILD_OF':
            o.constraints.remove(c)

    current_world = o.matrix_world.copy()
    c = o.constraints.new(type='CHILD_OF')
    c.target = arm
    c.subtarget = HEAD_BONE
    # Fórmula manual del inverse_matrix:
    # parent_mat^-1 @ current_world @ matrix_basis^-1
    c.inverse_matrix = parent_mat_inv @ current_world @ o.matrix_basis.inverted()

arm.data.pose_position = prev_pose  # vuelta a POSE
```

**Por qué fórmula manual y no el operator**: `bpy.ops.constraint.childof_set_inverse` con `temp_override` produjo resultados incorrectos (los objetos quedaban desplazados ~1.4m y rotados 90°). La fórmula manual `parent_mat^-1 @ current_world @ matrix_basis^-1` es la que aplica el operador internamente bien. Verificación: con esta fórmula el delta del objeto después de aplicar la constraint es exactamente `0.000000`.

**Por qué Child Of y no `parent_type='BONE'`**: el bone parenting cambia el `.parent` del objeto al armature, lo que rompe la jerarquía `Tiefling → eye/horn`. La constraint Child Of mantiene la jerarquía intacta y produce el mismo efecto visual.

### Pending issues conocidos

1. **Solo el Tiefling funciona con el armature compartido** porque está al world origin. `Human` (en X=2.5) y `Orc` (en X=5) no pueden activar el modifier Armature sin que sus cuerpos se "succionen" al origen donde está el armature. Soluciones posibles:
   - **(a)** Mover `Human` y `Orc` al origen y usar hide/show para alternar entre razas (estilo "costume swap")
   - **(b)** Crear armatures por raza, copiando la action — más overhead pero permite verlos lado a lado en la escena
   - **(c)** Linked Library Overrides — más complejo
2. **Root motion en Walking.fbx**: el Hips se desplaza en Y de `-0.09` (frame 1) a `-1.61` (frame 32) — el personaje camina hacia adelante en lugar de marcar el paso. Para sprite walk hay que limpiar las fcurves de location del Hips bone:
   ```python
   action = arm.animation_data.action
   to_remove = [fc for fc in action.fcurves
                if fc.data_path == 'pose.bones["mixamorig:Hips"].location']
   for fc in to_remove:
       action.fcurves.remove(fc)
   ```
3. **NPC sheets viejos (`entity_npc_orc.png`, 6 humanos)** usan frame order `SW, S, SE, E, NE, N, NW, W` — no matchea CLAUDE.md (`S, SW, W, NW, N, NE, E, SE`). Hay que regenerarlos cuando tengamos el render rig nuevo.
4. **Performance del viewport**: con Multires nivel 3 + Armature deformando, el viewport va lento. Para iterar más rápido sobre la animación se puede bajar `Multires.levels` (viewport) a 1 o 0 temporalmente — el render no se ve afectado porque usa `render_levels`.

---

## 7. Cómo agregar una nueva raza

Pasos para agregar una raza nueva al sistema de trajes (ej. `Elf`).

### Paso 1 — Duplicar el body master

```python
import bpy
src = bpy.data.objects["TF_Body_Sculpt"]  # master
new_body = src.copy()
new_body.data = src.data.copy()       # mesh data independiente (incluye multires layers)
new_body.data.name = "Body_Elf_Mesh"
new_body.name = "Body_Elf"
new_body.parent = None
new_body.matrix_parent_inverse.identity()
bpy.context.scene.collection.objects.link(new_body)
```

Heredás automáticamente: vertex groups `mixamorig:*`, modifier stack (Armature + Multires), todo el sculpt detail del master.

### Paso 2 — Materiales independientes

```python
for i, slot in enumerate(new_body.data.materials):
    new_mat = slot.copy()
    new_mat.name = f"{slot.name}_Elf"  # Body_Skin_Elf, Body_Top_Elf, Body_Bottom_Elf
    new_body.data.materials[i] = new_mat
```

### Paso 3 — Ojos independientes

```python
from mathutils import Matrix
iris_color = (0.20, 0.40, 0.10, 1.0)  # ej: verde

# Materiales de ojo
iris_mat = bpy.data.materials["TF_Eye_Iris"].copy()
iris_mat.name = "Eye_Iris_Elf"
iris_mat.use_nodes = True
iris_mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = iris_color
iris_mat.diffuse_color = iris_color

white_mat = bpy.data.materials["TF_Eye_White"].copy()
white_mat.name = "Eye_White_Elf"

# Objetos ojo
for src_name in ("TF_EyeR", "TF_EyeL"):
    src_eye = bpy.data.objects[src_name]
    new_eye = src_eye.copy()
    new_eye.data = src_eye.data.copy()
    new_eye.data.name = f"{src_name}_Elf_Mesh"
    new_eye.name = f"{src_name}_Elf"
    new_eye.parent = None
    new_eye.matrix_parent_inverse.identity()
    new_eye.matrix_world = src_eye.matrix_world.copy()  # misma posición que el original
    # Reemplazar slots
    for i, slot in enumerate(new_eye.data.materials):
        if slot is None: continue
        if slot.name.startswith("TF_Eye_Iris"):
            new_eye.data.materials[i] = iris_mat
        elif slot.name.startswith("TF_Eye_White"):
            new_eye.data.materials[i] = white_mat
    bpy.context.scene.collection.objects.link(new_eye)
```

### Paso 4 — Empty raíz del personaje y reparenting

```python
elf = bpy.data.objects.new("Elf", None)
elf.empty_display_type = 'PLAIN_AXES'
elf.empty_display_size = 0.4
bpy.context.scene.collection.objects.link(elf)
elf.location = (0.0, 0.0, 0.0)  # ver pending issue #1: idealmente al origen para que funcione el armature compartido
bpy.context.view_layer.update()

for n in ("Body_Elf", "TF_EyeR_Elf", "TF_EyeL_Elf"):
    o = bpy.data.objects[n]
    mw = o.matrix_world.copy()
    o.parent = elf
    o.matrix_parent_inverse = elf.matrix_world.inverted() @ mw @ o.matrix_basis.inverted()
```

### Paso 5 — Child Of constraints en los ojos

Aplicar el código de la sección 6 "Cómo attachear ojos / cuernos al head bone" sobre `TF_EyeR_Elf` y `TF_EyeL_Elf`. Si la raza tiene accesorios extra (cuernos, antenas, orejas separadas), repetir para cada uno.

### Paso 6 — Sculpt diferenciador (manual en Blender)

Seleccionar `Body_Elf` → Sculpt Mode. Como la mesh data es independiente (`Body_Elf_Mesh`), todo lo que sculptes acá no afecta a las demás razas.

Para un elfo: estirar las orejas a punta (ya las tiene si copiaste del tiefling — quizás suavizar la cara, hacer la barbilla más fina, etc.).

### Paso 7 — Setear paleta de colores

```python
def set_color(mat_name, color, roughness=None):
    m = bpy.data.materials[mat_name]
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = color
    if roughness is not None:
        bsdf.inputs["Roughness"].default_value = roughness
    m.diffuse_color = color  # ¡importante para que se vea en solid viewport!

set_color("Body_Skin_Elf",   (0.85, 0.75, 0.65, 1.0), roughness=0.6)
set_color("Body_Top_Elf",    (0.10, 0.30, 0.15, 1.0))
set_color("Body_Bottom_Elf", (0.05, 0.10, 0.05, 1.0))
```

> **Crítico**: siempre seteá `mat.diffuse_color` además de `Base Color` del BSDF. Si solo seteás Base Color, en solid viewport el material se ve blanco (default). Lo mismo para los materiales de eyes.

### Paso 8 — Verificar

1. Cambiar `arm.data.pose_position = 'POSE'`
2. Scrub el timeline → el body de la nueva raza se debería deformar igual que el tiefling
3. Verificar que los ojos siguen el head durante la animación
4. Verificar que el sculpt detail no se rompe (orden Armature → Multires)

### Checklist resumen

- [ ] Body duplicado con mesh data independiente
- [ ] Materiales `Body_Skin/Top/Bottom_<Race>` con `diffuse_color` seteado
- [ ] Ojos duplicados con mesh data + materiales `Eye_Iris/White_<Race>` independientes
- [ ] Empty `<Race>` creado al origen
- [ ] Body + ojos parented al empty preservando world transforms
- [ ] Stack del modifier del body es `[ARMATURE, MULTIRES]` (en ese orden)
- [ ] Child Of constraints en los ojos apuntando a `mixamorig:Head`
- [ ] Sculpt diferenciador hecho en Sculpt Mode
- [ ] Verificación visual: animación del armature deforma al body sin romper sculpt, ojos siguen head

---

## 8. Render de walk sprites (player)

### Configuración definitiva

El walk render usa la misma cámara y lighting que el idle, pero en POSE mode con la action del armature activa. Configuración validada:

| Parámetro | Valor |
|-----------|-------|
| Engine | EEVEE (Next) |
| Camera rotation | `(60°, 0°, 45°)` per CLAUDE.md |
| Camera type | Orthographic |
| `ortho_scale` | `1.69 * 1.05 / 0.9` ≈ 1.972 (tiefling = 10% más chico que orco) |
| Resolution | 128×256 (final) / 512×1024 (preview) |
| Sun energy | 1.10 (plus10 config) |
| Sun shadows | OFF |
| World bg strength | 9.90 (plus10 config) |
| Film transparent | True (RGBA PNG) |

### Tracking del Hips bone (CRÍTICO)

El cuerpo del tiefling tiene un offset fijo de ~0.84 en -Y respecto al origen (inherente a la pose de Mixamo, no del root motion). Para que body + ojos + cuernos + sombra queden centrados en el frame, hay que **centrar la cámara y la sombra en el Hips bone** en cada frame:

```python
hips = (arm.matrix_world @ arm.pose.bones["mixamorig:Hips"].matrix).translation
cam_target = Vector((hips.x, hips.y, 0.85))
cam.location = cam_target - look_dir * 10.0
shadow.location = (hips.x, hips.y, 0.001)
```

**NO mover el Tiefling empty ni parentar el Armature al empty.** Dejar ambos en identity. Los ojos y cuernos siguen al head bone via Child Of constraint (ACTIVO, no muteado).

### Rotación del personaje para las 8 direcciones

**NO se puede rotar el armature object** porque los bone data internos son Y-up (de Mixamo). Rotar el armature mueve los bones a posiciones incorrectas en world space.

Para idle (REST mode): rotar el Tiefling empty con Child Of muteados.
Para walk (POSE mode): **pendiente de resolver** — actualmente solo funciona la dirección SW (sin rotación).

> **TODO**: para las otras 7 direcciones del walk (S, W, NW, N, NE, E, SE), hay que investigar: (a) rotar los bones en edit mode a Z-up para que la rotación del armature funcione, o (b) rotar la cámara alrededor del personaje (menos limpio pero funcional), o (c) crear 8 copias del armature cada una rotada.

### Muestreo de frames de la animación

La action `Armature|mixamo.com|Layer0` tiene rango [1, 32] (32 frames de ciclo). Se muestrean 8 frames equiespaciados:

```python
SAMPLE_FRAMES = [1, 5, 9, 13, 17, 21, 25, 29]
```

### Proceso de render (paso a paso)

1. **Backup** sprites existentes en `assets/sprites/_backup/<timestamp>/`
2. **Preview**: renderizar el primer frame en alta resolución (512×1024) y abrirlo con el visor de imágenes de Windows para aprobación visual:
   ```python
   import subprocess
   subprocess.Popen(["start", "", preview_path], shell=True)
   ```
3. **Esperar aprobación** del usuario antes de continuar con el batch
4. **Batch render**: para cada dirección cardinal (S, SW, W, NW, N, NE, E, SE), para cada frame muestreado (0..7):
   - `scene.frame_set(action_frame)`
   - Leer Hips world XY → centrar cámara y sombra
   - Renderizar a `assets/sprites/player/walk/entity_player_walk_{cardinal}_{frame}.png`
5. **Restaurar** todo el estado de la escena

### Root motion

Las fcurves de location del Hips bone (`pose.bones["mixamorig:Hips"].location`) fueron eliminadas permanentemente de la action. Sin embargo, el Hips mantiene un offset fijo de ~0.84 en -Y inherente a la pose. El tracking de la cámara/sombra al Hips compensa este offset.

### Animated idle (TODO)

Mismo pipeline que walk pero con una animacion de "Breathing Idle" de Mixamo en vez de "Walking".

**Pasos:**

1. **Descargar FBX de Mixamo:** para cada body type (tiefling, orco, humano), exportar el body lowpoly a Mixamo, buscar "Breathing Idle" o "Idle", descargar el FBX con la animacion aplicada.

2. **Importar en Blender:**
   ```python
   bpy.ops.import_scene.fbx(
       filepath=r"C:\...\BreathingIdle.fbx",
       automatic_bone_orientation=True,
   )
   ```
   Igual que Walking.fbx. Eliminar el mesh generico de Mixamo que viene con el FBX.

3. **Limpiar root motion** (si lo tiene): eliminar fcurves de location del Hips bone, igual que se hizo para walk:
   ```python
   action = arm.animation_data.action
   to_remove = [fc for fc in action.fcurves
                if fc.data_path == 'pose.bones["mixamorig:Hips"].location']
   for fc in to_remove:
       action.fcurves.remove(fc)
   ```

4. **Configuracion de render:** usar el mismo `ortho_scale` que walk (el bbox consistente ya calculado cubre ambas animaciones — idle tiene menos movimiento que walk, asi que si walk entra, idle tambien entra). Misma camara, misma iluminacion, mismo resolution (128x256 o 256x512).

5. **Muestreo de frames:** misma logica que walk — 8 frames equiespaciados del ciclo de la action. Si la action tiene rango [1, N], muestrear `[1, 1+step, 1+2*step, ..., 1+7*step]` donde `step = N // 8`.

6. **Render batch:** para cada direccion (rotar pivote 0/45/90/.../315 grados), para cada frame muestreado, trackear Hips para centrar camara y sombra, renderizar PNG.

7. **Output:**
   - Player (tiefling): `assets/sprites/player/idle/entity_player_idle_{cardinal}_{frame}.png`
   - Orco: `assets/sprites/enemy/orc/entity_orc_idle_{cardinal}_{frame}.png`
   - NPCs humanos: para cada variante, cambiar materiales (skin/top/bottom) y renderizar a `assets/sprites/npc/{variant}/entity_npc_{variant}_idle_{cardinal}_{frame}.png`

8. **Diferencia clave con walk:** la animacion de breathing idle es mucho mas sutil (expansion de pecho, micro-shift de peso). Si los 8 frames muestreados se ven casi identicos, considerar usar un muestreo mas espaciado o una animacion con mas movimiento (ej. "Happy Idle" o "Weight Shift" de Mixamo).

---

## 9. Skill relacionada

`.claude/skills/render-npc-idle-spritesheet/SKILL.md` contiene un script Python parametrizable que genera un spritesheet completo para un body cualquiera. Este documento es la referencia conceptual; la skill es la herramienta. **La skill todavía tiene la fórmula vieja de ángulos** (anterior al fix de SW); cuando re-rendericemos los 8 frames con éxito desde el rig persistente, hay que actualizarla con el nuevo enfoque (rotar pivote en vez de cámara).
