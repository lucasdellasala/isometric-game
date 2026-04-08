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

## 2. Transparencia de objetos que tapan al player — `hecho`

Entidades (NPCs, enemies) cercanas al player se dibujan semi-transparentes cuando están delante, para que el jugador siempre sea visible.

**Regla actual:** Chebyshev distance ≤ 1 (las 9 celdas alrededor del player) + depth row mayor o mismo tile → alpha 128. Entidades más lejos o detrás quedan opacas.

**Implementación:**
- `draw_entity` recibe `player_depth` y compara contra `entity_depth_row(entity)`
- Muros (`draw_wall_cube`) usan `player_rect` pre-calculado para el mismo chequeo
- Pastos frontales usan `player_rect` con `has_intersection`

**Limitación conocida (mejora futura):** Para entidades de size=1 tile, el chequeo por depth row + proximidad funciona bien. Se intentó usar intersección de rects (`has_intersection` con rect completo, mitad inferior, tercio inferior) pero los sprites altos (128×256) generaban falsos positivos en las diagonales traseras. Si se agregan entidades de más de 1 tile, revisitar con bounding boxes más ajustados o colisión por píxel.

**Ubicación en código:** `render/renderer.rs` — `draw_entity`, `draw_wall_cube`, `draw_grass_tufts`. Sin tocar `core/`.

---

## 3. Editor de mapas in-game (place tiles, NPCs y spawn) — `pendiente`

Modo editor activable con una tecla (ej: `F2`) que permite construir mapas a mano sobre el mismo cliente del juego: pintar tiles, colocar NPCs/walls, marcar el spawn del jugador, y guardar/cargar a `assets/maps/*.json`. Reusa todo el rendering existente — la idea es que editar un mapa se vea exactamente igual que jugarlo.

**Modo Editor (toggle con F2):**
- Pausa la lógica del juego (`tick()` no corre, FOV se desactiva o se fuerza brightness=1.0).
- Muestra un overlay con: paleta de tiles, paleta de entidades, herramientas (Brush/Erase/Picker/SpawnMarker), nombre del mapa cargado, y atajos visibles.
- Cursor: el hover de tile actual se reusa, pero ahora resalta en color según la herramienta (verde=brush, rojo=erase, amarillo=spawn).
- Click izq = aplicar herramienta, click der = picker (toma el tile/entidad bajo el cursor como pincel).

**Paleta de tiles (izquierda):**
- Lista vertical scrolleable con thumbnails 64×32 de cada `TileKind` × variante disponible (Grass/Dirt/Stone/Water + WallObject).
- Click en un item = lo selecciona como pincel actual.
- Reusa los `Texture` que ya tiene `AssetManager`, no hace falta cargar nada nuevo.

**Paleta de entidades (derecha):**
- Lista de NPCs disponibles (los 9 variants ya generados) + Enemy + Player Spawn.
- Cada item es un thumbnail del frame 0 del spritesheet.
- Click = selecciona el "pincel de entidad". El siguiente click izquierdo en el grid coloca un NPC nuevo en ese tile (con un nombre autogenerado tipo `npc_{id}` y diálogo placeholder).
- Player Spawn es un caso especial: solo puede haber **uno**. Click lo mueve, no lo duplica. Se renderiza como un marcador `S` amarillo sobre el tile.

**Herramientas (top toolbar):**
- **Brush (B):** pinta el tile/entidad seleccionado en cada celda donde clickees (o donde arrastres).
- **Erase (E):** borra entidad si hay una en el tile, sino vuelve el tile a `Grass` por defecto.
- **Picker (P):** click en una celda copia su contenido al pincel actual.
- **Spawn (S):** marca esa celda como spawn del player.
- **Resize map:** popup con cols/rows nuevos, reescala el `Vec` de tiles preservando el contenido (si achicás, recorta; si agrandás, rellena con Grass).

**Save / Load:**
- `Ctrl+S` guarda el mapa actual a su path. Si es nuevo, abre un input de texto con el nombre.
- `Ctrl+O` muestra un listado de los `.json` en `assets/maps/` y permite cargar uno.
- El formato JSON ya existente (`Tilemap` con `tiles`, `entities`, `walls`) se extiende con un campo opcional `"player_spawn": [col, row]`. Si falta, se asume `(cols/2, rows/2)` como hoy.
- Al cargar/guardar, el `GameState` se recrea desde cero con el nuevo `Tilemap` para evitar estados inconsistentes.

**Integración con el game loop:**
- `main.rs` agrega un nuevo `GameMode { Playing, Editing }`. En modo Editing, el input se rutea al editor en vez de a `apply_input()`.
- El editor vive en `src/ui/map_editor.rs` con estado propio (`EditorState { tool, brush, hover, dirty }`). Sin tocar `core/`.
- `GameState::from_tilemap_with_spawn(tilemap, spawn)` — método nuevo en `core/game_state.rs` que construye una `GameState` con un spawn explícito en vez del centro hardcoded.
- El `tilemap.rs` gana un método `set_tile(col, row, kind)` y `add_entity_at(col, row, kind, variant)` que el editor llama mientras pinta.

**Persistencia mientras editás (autosave):**
- **Cada modificación se persiste inmediatamente** al archivo `assets/maps/_editor_autosave.json`. No hay riesgo de perder cambios si crashea o si salís sin guardar — el estado del editor siempre coincide con lo que hay en disco.
- El autosave se hace al cerrar la operación atómica (terminar un trazo de brush, soltar el click, mover el spawn, resize, etc.), no en cada pixel del drag, para no martillar el disco. En la práctica: cada `EditorAction` ejecutada → marshal del `Tilemap` completo a JSON → write atómico (write a temp file + rename) para no dejar el archivo medio escrito si crashea durante el write.
- `Ctrl+S` sigue existiendo, pero ahora sirve para **"guardar como"** en un nombre custom (ej: `forest_v2.json`) además del autosave. El autosave queda como red de seguridad permanente.
- Al iniciar el juego en modo Editor, si existe `_editor_autosave.json`, se carga automáticamente. Si no existe, se carga el último mapa que estuvo abierto (guardado en `assets_dev/_editor_state.json` con el path del último mapa) o el mapa default del juego.
- `dirty` flag sigue existiendo pero solo refleja "hay cambios desde el último Ctrl+S a un nombre custom", no desde el último autosave. Útil solo para mostrar un asterisco `*` en el título cuando estás en un mapa con nombre y todavía no lo guardaste explícitamente con ese nombre.
- No hay undo/redo en el v1 — agregalo después si lo necesitás.

**Casos a manejar:**
- Tile bajo el cursor con una entidad encima: Brush con tile = solo cambia el tile, mantiene la entidad. Brush con entidad = reemplaza la entidad anterior. Erase borra primero entidad, segundo click borra el tile.
- Player Spawn sobre un tile inválido (Water/Wall): el editor lo permite pero al guardar muestra un warning.
- Resize que deja el spawn fuera del nuevo tamaño: el editor lo reposiciona al centro automáticamente con un toast.

**Cargar mapas desde el menú de debug (modo Playing también):**
- Nuevo submenú **"Maps"** en el debug menu unificado (TAB), accesible tanto en modo Playing como en modo Editing.
- Lista todos los `.json` encontrados en `assets/maps/` (escaneados con `std::fs::read_dir`), excluyendo `_editor_autosave.json` y otros archivos que empiecen con `_`.
- Cada item muestra: nombre del mapa, dimensiones (`64×64`), cantidad de entidades, y un indicador `(actual)` si es el que está cargado.
- **Enter** sobre un item carga ese mapa: recrea el `GameState` desde cero con `GameState::from_tilemap_with_spawn()`, resetea cámara al spawn del player, limpia diálogos abiertos y dispara un `GameEvent::MapChanged { map_name }`.
- **Atajo "Reload current map" (R)** dentro del submenú: recarga el archivo desde disco, útil cuando editaste el JSON con un editor externo o cuando estás iterando entre Editor → Playing → Editor.
- **Atajo "Open editor autosave"**: shortcut directo para cargar `_editor_autosave.json` y seguir testeando lo último que estuviste editando.
- En modo Playing, cargar otro mapa pierde el estado del personaje (posición, FOV explorada, diálogos vistos) — para v1 es aceptable porque no hay save/load todavía. En P7 (save/load) esto se integra mejor.
- En modo Editing, cargar otro mapa primero hace flush del autosave actual del mapa que estás editando bajo su nombre real (no `_editor_autosave.json`) si tiene nombre asignado, para no perderlo.

**Ubicación en código:**
- Nuevo `src/ui/map_editor.rs` con `EditorState`, `update()`, `draw_overlay()`.
- Nuevo `src/ui/editor_palette.rs` opcional si la lógica de paletas crece.
- `src/main.rs`: agregar `GameMode` enum, branch en input loop según modo.
- `src/core/tilemap.rs`: campo opcional `player_spawn`, métodos `set_tile`/`add_entity_at`, función `save_to_json(path)`.
- `src/core/game_state.rs`: constructor `from_tilemap_with_spawn`.
- Sin tocar `render/renderer.rs` salvo para que `draw_entities_and_ui()` invoque al overlay del editor cuando `mode == Editing`.

**Por qué hacer esto antes que P10 (Map Editor visual standalone):** este es un editor *in-game* mucho más simple, no requiere herramientas externas, y desbloquea la creación de contenido para P5-P8 (combat, dialogue, story) sin tener que escribir mapas a mano en JSON. P10 puede ser un editor más sofisticado más adelante (con tooling visual completo, batch operations, prefabs).

---

## 4. Voces para diálogos de NPCs (texto + audio sincronizado) — `pendiente`

Sistema de diálogo estructurado con líneas múltiples y, opcionalmente, un archivo de voz asociado a cada línea. Cuando el jugador interactúa con un NPC y avanza por una línea, suena el clip correspondiente y el texto aparece sincronizado en el cuadro de diálogo. Si no hay audio para esa línea, el sistema cae a "modo silencioso" con tiempo de lectura proporcional al largo del texto.

**Estructura del diálogo (data, no código):**

Hoy un NPC tiene un único string en `Entity.dialogue`. Hay que evolucionar a una estructura tipo árbol/lista. Formato JSON propuesto, vive junto al mapa o como archivo aparte en `assets/dialogues/{npc_id}.json`:

```json
{
  "id": "guard_intro",
  "speaker": "Guardia",
  "lines": [
    { "id": "01", "text": "Alto ahí, viajero.",                    "voice": "guard_intro_01.ogg" },
    { "id": "02", "text": "¿Qué te trae a estas tierras malditas?", "voice": "guard_intro_02.ogg" },
    { "id": "03", "text": "...",                                    "voice": null }
  ]
}
```

- `id` por línea = nombre estable para que el archivo de voz no se rompa si reescribís el texto.
- `voice` opcional. Si es `null` o el archivo falta, se usa fallback silencioso.
- Más adelante (P8 dialogue branching) este formato se extiende con `choices`, `next`, `condition`, `flag_set`, etc. Por ahora, lista lineal.

**Formato de los archivos de audio:**

- **Codec:** `OGG Vorbis` (libre, comprimido, soportado nativamente por SDL2_mixer y por `rodio`. Evita MP3 por patentes históricas y .wav por tamaño).
- **Sample rate:** `22050 Hz` mono. Para voz humana es más que suficiente (Nyquist cubre hasta 11kHz, la voz hablada vive en 80Hz–8kHz). Stereo no aporta nada para diálogos sin ambiente.
- **Bitrate:** `~64 kbps` VBR (calidad ~3 en oggenc). Una línea de 3 segundos pesa ~24 KB. Mil líneas pesan ~24 MB, manejable.
- **Loudness target:** `-18 LUFS` para que todas las líneas tengan volumen consistente. Normalizar con ffmpeg en batch como parte del pipeline de assets.
- **Convención de nombre:** `{dialogue_id}_{line_id}.ogg` (ej: `guard_intro_01.ogg`). Snake case, sin espacios ni acentos.
- **Ubicación:** `assets/voices/{npc_id}/`. Una carpeta por NPC para no inundar un solo directorio. Ej: `assets/voices/guard/guard_intro_01.ogg`.

**Pipeline de generación de voces (dev-time, no runtime):**

- Para v1 alcanza con grabarlas a mano o usar TTS local (ej: piper, mimic3, ElevenLabs si hay presupuesto). El proyecto no necesita generar audio en runtime.
- Script en `assets_dev/scripts/gen_voices.py` que toma el JSON del diálogo, llama al motor TTS configurado, y exporta cada línea como `.ogg` normalizado a -18 LUFS con ffmpeg. Idempotente: si el archivo ya existe y el texto del JSON no cambió (hash en metadata), lo skipea.
- Los `.ogg` finales se commitean a `assets/voices/`. El script y los modelos TTS quedan en `assets_dev/` (no se commitean).

**Sistema de runtime — `AudioManager`:**

Nuevo módulo `src/render/audio.rs` (o `src/audio.rs` si crece y queremos sacarlo de render).

- **Dependencia:** agregar feature `mixer` al crate `sdl2` en `Cargo.toml`. Eso da acceso a `sdl2::mixer` que carga `.ogg` directamente. Ya estamos usando SDL2 para todo, mantiene la dependencia única. Alternativa: `rodio` (pure Rust, sin libs nativas) — más portable pero agrega un crate grande. **Default propuesto: `sdl2::mixer`** porque ya pagamos el costo de las libs nativas con SDL2.
- `AudioManager::new()` inicializa el mixer (`sdl2::mixer::open_audio(22050, AUDIO_S16LSB, 2, 1024)`), carga channels (8 para SFX + 1 dedicado para voz).
- `AudioManager::play_voice(path: &str)` carga el `.ogg` en un `Chunk`, lo reproduce en el canal de voz, devuelve un `VoiceHandle` con `is_playing()` y `stop()`.
- `AudioManager::play_sfx(name)` para futuros efectos (P4 completo).
- Cache de chunks ya cargados en un `HashMap<String, Chunk>` para no releer del disco si se repite una línea.

**Sistema de runtime — `DialogueState`:**

Hoy `ActiveDialogue` guarda solo `entity_id` y `text`. Hay que extenderlo:

```rust
pub struct ActiveDialogue {
    pub entity_id: u64,
    pub speaker: String,
    pub lines: Vec<DialogueLine>,
    pub current_line: usize,
    pub line_started_at_tick: u64,
    pub voice_handle: Option<VoiceHandle>,
}
```

Flujo:
1. Player presiona `E` cerca de un NPC → `GameInput::Interact { entity_id }`.
2. `GameState::apply_input()` carga el JSON del diálogo asociado al NPC (`assets/dialogues/{npc.dialogue_id}.json`), crea `ActiveDialogue`, dispara `GameEvent::InteractionStarted { dialogue_id }`.
3. El renderer (que sí conoce el `AudioManager`) reacciona al `GameEvent`: lee `lines[0]`, llama `audio.play_voice("assets/voices/{dialogue_id}/{dialogue_id}_01.ogg")`, guarda el `VoiceHandle` en el `ActiveDialogue` (vía un puente — ver siguiente bullet).
4. El cuadro de diálogo dibuja `lines[current_line].text` con el speaker.
5. Player presiona `E` o `Espacio` para avanzar:
   - Si el `voice_handle` actual sigue sonando, primero la corta (skip). Si ya terminó, avanza a `lines[current_line + 1]`.
   - Si llegó al final, cierra el diálogo (`GameEvent::InteractionEnded`).

**Cómo cruzar el Golden Rule sin romperlo:**

`GameState` no puede conocer al `AudioManager` (rompe el principio "core puro, no SDL2"). Dos opciones:

- **(A) GameEvent disparado por GameState, audio reaccionando en main:** `GameState.apply_input` solo emite `GameEvent::DialogueLineAdvanced { line_id, voice_path }`. El game loop en `main.rs` lo lee del `Vec<GameEvent>` y llama `audio.play_voice()`. El `voice_handle` queda guardado en una struct render-side (`DialogueRenderer`), no en `GameState`. **Esta es la opción que mantiene el Golden Rule limpio.**
- **(B) Trait `AudioBackend` inyectado:** `GameState` recibe un `&mut dyn AudioBackend` en `apply_input`. Más flexible para tests pero introduce trait objects (`dyn Trait` aún no aprendido — ver sección "concepts not yet learned" en CLAUDE.md). Postergar a P9 cuando ya haya razón para abstraer el cliente vs servidor.

→ **Default: opción (A).** Postergar (B) hasta que sea necesario.

**Subtítulos y timing en modo silencioso:**

- Si `voice == null` o el archivo no existe, calcular `display_duration_ticks = max(60, text.len() * 3)` (~3 ticks por carácter, mínimo 1 segundo). El jugador puede saltearlo con E.
- Si hay audio, `display_duration_ticks` se ignora — la línea termina cuando termina el audio (o cuando el jugador la skipea con E).
- Mostrar el texto completo de una vez (no typewriter en v1). Typewriter viene en P8 si lo querés.

**Volumen y mute:**

- Agregar al menú de debug (TAB → Game Settings) sliders para `master_volume`, `voice_volume`, `sfx_volume` (0-100, default 80/100/80). Persistir a un `settings.json` en P7 cuando llegue save/load.
- Tecla `M` toggle global mute.

**Casos a manejar:**
- Player abre diálogo y se aleja del NPC mid-line → diálogo se cierra y voz se corta.
- Player abre diálogo de un NPC mientras otra voz suena (ej: música ambient con voz, o un narrador) → cortar la anterior antes de empezar la nueva. Solo un canal de voz activo.
- Archivo `.ogg` corrupto o falta → log de warning, fallback a modo silencioso, no crashear.
- Diálogo con `lines: []` vacío → cerrar inmediatamente con un warning.

**Ubicación en código:**
- Nuevo `src/render/audio.rs` con `AudioManager`, `VoiceHandle`, init en `main.rs`.
- Nuevo `src/core/dialogue.rs` con `Dialogue`, `DialogueLine`, `load_from_json()` (sin tocar SDL2 — solo serde + structs).
- Modificar `src/core/game_state.rs` `ActiveDialogue` para soportar múltiples líneas.
- Modificar `src/core/input.rs`: agregar `GameEvent::DialogueLineAdvanced { dialogue_id, line_id, voice_path }` y `GameEvent::DialogueEnded`.
- Modificar `src/main.rs` para reaccionar a estos eventos llamando al `AudioManager`.
- Modificar `src/render/renderer.rs` `draw_dialogue_box()` para mostrar el speaker + texto de la línea actual.
- Modificar `assets/maps/map.json` para que las entidades referencien `"dialogue_id": "guard_intro"` en vez del string inline actual.
- Nuevo directorio `assets/dialogues/` con los JSON.
- Nuevo directorio `assets/voices/{npc_id}/` con los `.ogg`.
- Nuevo `assets_dev/scripts/gen_voices.py` para el pipeline de generación.

**Por qué hacerlo separado de P4 (Audio + polish):** P4 es un cajón grande. Esto es un slice acotado y útil por sí mismo: dialogue + voice acting. Si lo cerrás antes, después agregás música/SFX/particles encima sin tener que pensarlo todo junto.

---

## 5. Pathfinding visualizado (debug view) — `pendiente` — **URGENTE**

Toggle en el debug menu que pinta visualmente el A* corriendo: `open set` en azul semi-transparente, `closed set` en gris, path final en verde, nodo actualmente expandido en amarillo. Es 100% debugging y **client-local**: en coop (P9) cada jugador ve solo el path de *su* personaje cuando activa su propio debug, no se sincroniza por red. Va a ser invaluable cuando lleguen los enemigos con AI en P6 y necesitemos entender por qué un mob eligió cierta ruta.

**Implementación (respetando Golden Rule — no mete debug en `GameState`):**
- `core/pathfinding.rs::find_path()` gana una variante `find_path_with_debug(&Tilemap, start, goal, &mut PathDebugInfo) -> Option<Vec<Pos>>` donde `PathDebugInfo` es un struct simple (también en `core/pathfinding.rs`) que el caller provee. La función original `find_path()` queda intacta para gameplay, solo agregamos un wrapper que captura estado.
- El renderer guarda su propio `PathfindingDebug { enabled: bool, last_path: PathDebugInfo, target_entity_id: u64 }` en una struct render-side (al lado de `AssetManager`/`AudioManager`/`DebugMenu`), **no en `GameState`**.
- Cuando el debug está activo, el renderer llama `find_path_with_debug()` directamente sobre `gamestate.tilemap` para *su* personaje, captura el resultado en su `PathfindingDebug` local, y lo dibuja como overlay.
- En coop (P9), cada cliente tiene su propia instancia de `PathfindingDebug` local. El host nunca envía debug por red. Cada jugador ve solo lo suyo.
- Toggle en el debug menu (TAB → Game Settings → "Show pathfinding debug").

---

## 6. Sistema de partículas básico (hit, polvo, magia) — `pendiente` — **P4**

Particle pool con structs simples (`Particle { pos, vel, life, color, size }`) que **vive 100% en `render/`** — las partículas no afectan gameplay (no cambian HP, no bloquean movimiento, no se sincronizan por red), así que no tienen razón de existir en `GameState`. Emisores disparados por `GameEvent` (`EntityHit` → chispas, `EntityMoved` sobre Dirt → polvo, hechizos → estelas). Sienta las bases visuales para combate (P6) y polish general.

**Importante (separación de ticks):** hay dos ticks distintos en el juego, no confundirlos:
- **Game tick** (`GameState.tick()` en `core/`): fixed timestep 60 Hz, lógica autoritativa, se sincroniza en P9.
- **Particle tick** (`ParticleSystem.update(dt)` en `render/`): corre al framerate del cliente, decrementa lifetimes, descarta partículas muertas. Es solo visual.

**Implementación:**
- Nuevo `src/render/particles.rs` con `ParticleSystem { pool: Vec<Particle> }` (capacidad fija con compactación al final del frame para evitar allocations).
- `main.rs` guarda el `ParticleSystem` al lado del `AudioManager` y `AssetManager`.
- Después de `state.tick()`, el game loop itera el `Vec<GameEvent>` que devolvió el tick y por cada `EntityHit/EntityMoved/SpellCast/...` llama `particles.emit(...)` con la posición y tipo del emisor.
- Cada frame antes de renderizar: `particles.update(dt)`, después `particles.draw(&mut canvas)`.
- Core nunca sabe que existen partículas. En P9, las partículas son puramente locales del cliente — el host no las simula ni las envía.

---

## 7. Ciclo día/noche con luz dinámica — `pendiente` — **P4**

Variable `time_of_day: f32` (0.0–24.0) en `GameState` que avanza X unidades por tick. El renderer aplica un `set_color_mod` global a tiles y entidades según una `ColorRamp` por hora (medianoche=azul oscuro, amanecer=naranja cálido, mediodía=blanco puro, atardecer=ámbar). El FOV radius también puede variar (más corto de noche), creando gameplay emergente sin agregar mecánicas nuevas.

**Ubicación:** Campo nuevo en `core/game_state.rs`, función `time_to_color_mod()` en `render/renderer.rs` aplicada en `draw_tiles()` y `draw_entity()`.

---

## 8. Sonidos ambientales por bioma — `pendiente` — **P4**

Loops de audio ambient (viento, río, antorchas, cueva) que arrancan/paran según el `TileKind` bajo el player, con crossfade suave de ~1s. `AudioManager::set_ambient(biome_id)` corta el loop anterior y arranca el nuevo en un canal dedicado. Cada `TileKind` tiene un `ambient_id: Option<&'static str>` definido en una tabla en `core/tilemap.rs`.

**Ubicación:** Extensión del `AudioManager` de la idea #4. Tabla de mappings tile→ambient en `core/tilemap.rs`. `main.rs` chequea cada N ticks el tile bajo el player y llama `audio.set_ambient()` si cambió.

---

## 9. Pickup de items en el mundo — `pendiente` — **P5**

Nueva variante `EntityKind::Item { item_id: String, quantity: u32 }` con sprite propio cargado desde `assets/sprites/items/`. Al caminar sobre el tile (o presionar E adyacente), se dispara `GameEvent::ItemPickedUp { item_id, quantity }` y la entidad se elimina del mundo. En v1 el item solo se "consume" sin guardarse en ningún lado — sienta las bases del inventario completo de P5 sin tener que esperar al sistema entero.

**Ubicación:** Variante en `core/entity.rs::EntityKind`, lógica de pickup en `game_state.rs::tick()`, sprites en `assets/sprites/items/`, render en `draw_entity()` con un pequeño bob vertical para que se note que es interactuable.

---

## 10. Mini-mapa en esquina superior derecha — `pendiente` — **polish (sin fase)**

Render de una versión miniatura del mapa (1px por tile) con colores base de cada `TileKind`, posición del player como punto blanco parpadeante, NPCs como puntos amarillos, área explorada vs no explorada según FOV (lo no explorado se dibuja en negro). Toggle con tecla `M`. Útil para debug ahora y para gameplay real cuando los mapas crezcan a 200×200+.

**Ubicación:** Nuevo `src/ui/minimap.rs` con `draw_minimap(canvas, gamestate, screen_w, screen_h)` llamado al final de `draw_entities_and_ui()`. Implementación con `canvas.draw_point()` o un buffer pre-renderizado actualizado cada N ticks.

---

## 11. Indicadores de daño flotantes (floating damage numbers) — `pendiente` — **P6**

Texto efímero que sale del entity al recibir daño/curar, sube unos pixels en arco y se desvanece en ~1 segundo. Pool similar al de partículas (#6), reusa el `TextRenderer` ya existente. Color por tipo: rojo para damage, verde para heal, amarillo para crit, azul para mana — feedback visual instantáneo y "juicy" que mejora muchísimo el feel del combate antes de tener mecánicas profundas.

**Ubicación:** Nuevo `src/render/floating_text.rs` con `Vec<FloatingText>` y `update()/draw()`. Disparado por `GameEvent::EntityDamaged { amount, kind }` desde el game loop.

---

## 12. Slash/efecto direccional al atacar — `pendiente` — **P6**

Sprite animado de un slash (3-5 frames) que se dibuja una sola vez en la dirección que mira el player cuando ataca. Emitido por `GameEvent::PlayerAttacked { direction, position }`, vive ~150ms y se autoelimina. No necesita combate funcional todavía — el visual solo ya es suficiente para iterar feel y feedback antes de meter HP/daño/initiative.

**Ubicación:** Sprites de slash en `assets/sprites/effects/slash_*.png`. Pool de efectos one-shot en `src/render/effects.rs` con duración y dirección por instancia.

---

## 13. Hover info sobre NPCs y objetos — `pendiente` — **polish (sin fase)**

Cuando el mouse está sobre una entidad, aparece un tooltip pequeño con su nombre, kind, y (más adelante) HP/level/faction. Implementado reusando el sistema de hover de tile que ya existe en `render/renderer.rs`. Para NPCs adyacentes al player, agrega "Press E to talk" como hint de UX, eliminando la fricción de no saber qué hacer con un NPC nuevo.

**Ubicación:** Función nueva `draw_entity_tooltip()` en `render/renderer.rs` invocada después de `draw_entities_and_ui()`. Detección de entity bajo el cursor con un loop sobre `gamestate.entities` y check de bbox en pantalla.

---

## 14. Quicksave / Quickload (F5 / F9) — `pendiente` — **P7**

Quicksave con `F5` que serializa el `GameState` completo a `assets/saves/quick.json` (player pos, todas las entidades con sus estados, FOV explorada, diálogos vistos, time_of_day, etc.). Quickload con `F9` lo restaura. Sienta las bases del save/load completo de P7 sin esperarlo, y elimina la fricción de re-iniciar el juego cada vez que querés probar algo concreto.

**Modelo de save en coop (P9):** la partida guardada vive **en el host**. Si el host hace quicksave y se desconecta, todos los clientes se desconectan también — la partida no puede continuar sin él. Cuando un cliente se reconecta, recibe el `GameState` actual del host (los mismos bytes que se guardarían a disco, pero por socket en vez de a archivo). Mismo struct, mismos bytes, distinto transport.

**Implementación (respetando Golden Rule — separar "qué se serializa" de "a dónde van los bytes"):**
- `core/` agrega solo `#[derive(Serialize, Deserialize)]` a todos los structs (`GameState`, `Entity`, `Tilemap`, `FovMap`, etc.). **Nada más en core** — ningún método de I/O.
- `main.rs` (que es el "host" en solo player y será el host del coop en P9) hace el `fs::write`:
  ```rust
  fn quicksave(state: &GameState) -> io::Result<()> {
      let json = serde_json::to_string(state)?;
      fs::write("assets/saves/quick.json", json)
  }
  fn quickload() -> io::Result<GameState> {
      let json = fs::read_to_string("assets/saves/quick.json")?;
      Ok(serde_json::from_str(&json)?)
  }
  ```
- Tecla F5/F9 manejada en `main.rs` directamente sin pasar por `GameInput` (es una operación de cliente, no un input de juego).
- En P9, el mismo `serde` derive sirve para mandar el state por TCP con `bincode::serialize()`. Core no crece métodos por cada destino nuevo (`save_to_file`, `send_to_socket`, `compute_hash`, etc.) — el transport vive afuera siempre.
