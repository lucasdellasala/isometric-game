# IDEAS.md

Ideas de features y mejoras para el juego. Cada idea tiene un estado y una descripción breve de cómo implementarla. Sirve como backlog informal — cuando arranques una idea, cambiá el estado a "en progreso" y cuando esté lista a "hecho".

**Estados:** `pendiente` | `en progreso` | `hecho` | `descartado`

---

## 1. Pasto decorativo sobre tiles de Grass — `hecho`

Generar sprites de pasto orgánicos con Python/PIL y distribuirlos pseudo-aleatoriamente sobre cada tile Grass.

**Sprites de pasto:** PNGs de 16x24px con fondo transparente. Cada sprite tiene 3-5 briznas generadas proceduralmente: arrancan anchas en la base (1-2px), terminan en punta arriba, con inclinación y curvatura aleatoria. Color varía entre #3a5c2e y #5a8040. 8 variantes (`grass_tuft_01.png` a `grass_tuft_08.png`) en `assets/sprites/decorations/`.

**Oclusión parcial:** Los pastos con offset_y menor al centro del tile se dibujan antes de la entidad (quedan detrás), los de offset_y mayor se dibujan después (tapan los pies del personaje). El render loop por depth row pasa de 2 pasos (tiles → entidades) a 4: tiles → pastos traseros → entidades → pastos delanteros.

**Seed determinístico:** `col * 7919 + row * 6271` para que el patrón sea siempre igual sin guardar datos extra.

**Ubicación en código:** Nuevo `render/decorations.rs` con `generate_grass()` y `draw_grass_tufts()`. Sin tocar `core/`.

---

## 2. Transparencia de objetos que tapan al player — `pendiente`

Cuando un sprite se dibuja **delante** del player en el depth order y su rectángulo en pantalla intersecta con el del player, se dibuja semi-transparente (alpha 128) para que el jugador siga siendo visible detrás de muros, NPCs, árboles, etc.

**Regla:** Solo afecta sprites con depth row >= player depth row. Sprites detrás del player (depth row menor) siempre se dibujan opacos.

**Evaluación por sprite individual:** Cada tile, NPC, pasto o muro se evalúa de forma independiente. En una pared de 10 tiles, solo los 1-2 que realmente intersectan con el Rect del player se hacen transparentes. Los demás quedan opacos.

**Implementación:**
1. Al dibujar al player en `draw_entity`, guardar su `Rect` en pantalla y su `depth_row` en una variable accesible por el render loop.
2. Para todo sprite dibujado después (depth row >= player depth row): `if sprite_rect.has_intersection(player_rect) { texture.set_alpha_mod(128) } else { texture.set_alpha_mod(255) }`.
3. Aplica a: tiles de muro, NPCs, enemigos, pastos delanteros, y cualquier objeto futuro.

**Ubicación en código:** Modificar `draw_entity` y `draw_grass_tufts` en `render/renderer.rs`. Sin tocar `core/`.
