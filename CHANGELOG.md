# CHANGELOG

Registro de cambios del RPG isometrico multijugador estilo D&D, construido desde cero en Rust con SDL2.

---

## [Unreleased] — En desarrollo

### Agregado
- **`config.rs` centralizado:** Todas las constantes de configuración (colores, tamaños, velocidades, umbrales) viven en `src/config.rs`. Ningún valor visual o de gameplay se hardcodea inline
- **Highlight de interacción con NPCs:** Contorno exterior pre-calculado al cargar (detección de bordes en los PNGs) en verde para NPCs amigables y rojo para enemigos. Color uniforme independiente del color del sprite. Se muestra al pasar el mouse (hover) o al estar adyacente (Chebyshev ≤ 1). Prompt `[E] Hablar` cuando se puede interactuar
- **Transparencia de oclusión:** Entidades cercanas al player (Chebyshev ≤ 1) con depth row mayor o mismo tile se dibujan semi-transparentes (alpha 128). Muros y pastos usan intersección de rects con player_rect pre-calculado. Enemies comparten el sistema con NPCs
- **Pasto decorativo:** 8 sprites de hierba distribuidos pseudo-aleatoriamente sobre tiles Grass con oclusión parcial (detrás/delante del player)
- **Posiciones bloqueadas:** `GameState.blocked: HashSet<(i32,i32)>` para objetos que bloquean movimiento y pathfinding
- **NPCs y enemies con PNGs individuales por dirección:** 6 variantes de NPC + 1 de enemy (orc), 8 PNGs por variante. En runtime se cargan como spritesheets pre-generados para optimizar boot time
- **Idle breathing animations:** Animación sutil de respiración para todas las entidades (player, NPCs, enemies). 8 frames por dirección a ~2.5 FPS. Cada entidad arranca en un frame random para evitar sincronización
- **Entity shadow:** Sprite de sombra (`entity_shadow.png`) renderizado debajo de cada entidad con escala y offset configurables
- **Per-entity scale:** Escala por tipo de entidad (`SCALE_PLAYER`, `SCALE_NPC`, `SCALE_ENEMY_ORC`) para que el orc se vea más grande que los humanos sin cambiar el tamaño de PNG
- **Spritesheet build pipeline:** Script Python (`scripts/build_spritesheets.py`) que combina PNGs individuales en spritesheets (8 columnas × N filas). Solo regenera si los PNGs son más nuevos que el sheet
- **Debug menu: Entity Scale:** Submenu para ajustar en runtime: base scale, per-type scale, shadow scale/offset, walk/idle animation speed (ticks/frame)
- **Debug settings export:** Al cerrar el juego se exporta `debug_settings.json` con todos los valores actuales del debug menu
- **Boot timing logs:** `boot_timing.log` y `boot_assets.log` con breakdown detallado del tiempo de carga. Se acumulan entre ejecuciones para comparar optimizaciones
- **Movimiento 8-direccional:** WASD para 4 cardinales + combos (W+D, D+S, S+A, A+W) para diagonales
- **Pathfinding 8-direccional:** A* con costo cardinal=10, diagonal=14 (≈√2×10) y heurística octile
- **Direction enum:** Reemplaza `facing: u16` con ángulos por `facing: Direction` con cardinales de pantalla (N, NE, E, SE, S, SW, W, NW)
- **Pathfinding debug overlay:** Toggle en debug menu (Game Settings → Show pathfinding). Visualiza closed set (azul), path (verde), start (azul) y goal (amarillo)
- **Globo de diálogo (speech bubble):** Rectángulo redondeado con flecha apuntando al NPC, text wrapping automático, nombre del hablante y hint `[E] Cerrar`
- **Rotacion idle de NPCs:** los NPCs cambian de direccion aleatoriamente cada 3-8 segundos cuando estan quietos
- **NPCs miran al jugador al interactuar:** `face_toward()` orienta al NPC hacia el jugador al presionar E
- **Sistema de decoraciones (grass tufts):** 8 sprites de pasto decorativo con distribucion procedural por tile (0-3 tufts por tile con pesos), separados en capa trasera (detras de entidades) y capa frontal (delante de entidades)
- **Variantes de tiles por posicion:** funcion `noise()` determinista que elige entre 3 variantes para grass, dirt y stone segun la posicion del tile, con distribucion ponderada (60%/30%/10%)
- **18 variantes de agua animada:** soporte para 18 sprites de agua con selector en el menu de debug
- **Tiles de 128x64 (spritesheet pre-extraidos):** soporte para tiles de mayor resolucion desde spritesheets de forest, water y terrain
- **Menu de debug unificado:** reemplaza `ConfigMenu` y `SpriteDebug` con un solo `DebugMenu` con submenus navegables (Tab para abrir, flechas + Enter para navegar, Escape para volver)
  - Submenu Post-Process: modo (off/dither/moebius), scope, colores, spread, posterize, edge threshold
  - Submenu Sprite Offset: offset base y per-direction para sprites del jugador
  - Submenu Tile Preview: selector de variante de agua
  - Submenu Game Settings: radio FOV y zoom de camara
- **Zoom dinamico desde menu de debug:** zoom de camara configurable en tiempo real (0.5x a 4.0x), reemplaza la constante `CAMERA_ZOOM`
- **Radio FOV configurable:** `fov_radius` movido a `GameState` como campo publico, ajustable desde el menu de debug (5-40 tiles)
- **Dependencia `rand`:** agregada al `Cargo.toml` para variantes aleatorias de NPCs y rotacion idle

### Cambiado
- **Reorganizacion de assets:** los sprites se movieron de `assets/tiles/CelShading/` a una estructura organizada:
  - `assets/sprites/player/idle/` y `assets/sprites/player/walk/` para el jugador
  - `assets/sprites/npc/` para PNGs individuales de NPCs (8 por variante)
  - `assets/sprites/enemy/` para PNGs individuales de enemigos (8 por variante)
  - `assets/sprites/decorations/` para decoraciones
  - `assets/tiles/forest/`, `assets/tiles/water/`, `assets/tiles/terrain/` para tiles de terreno
  - `assets/maps/` para archivos JSON de mapas
  - `assets/fonts/` para fuentes TTF
- **Centrado de camara:** la camara ahora centra verticalmente en `screen_h / 2` en vez de `screen_h / 4`
- **Spawn del jugador al centro del mapa:** en vez de (0,0), ahora aparece en `(cols/2, rows/2)`
- **Tile rendering normalizado:** `draw_tile()` siempre dibuja a `TILE_WIDTH x TILE_HEIGHT` independientemente del tamano real del sprite
- **Frustum culling ampliado:** margen de culling duplicado a `TILE_WIDTH * 2` para evitar pop-in con zoom
- **Entity rendering con spritesheets:** PNGs individuales en disco, combinados en spritesheets por `scripts/build_spritesheets.py` para carga rápida. El renderer usa `src_rect` para recortar frames
- **Sprites a 256×512:** Resolución duplicada (antes 128×256). `ENTITY_SCALE = 0.33` compensa
- **Interacción con NPCs a 8 direcciones:** Chebyshev distance ≤ 1 (antes solo 4 cardinales)
- **Outlines solo para idle estáticos:** Walk/idle animados usan el outline del idle estático (imperceptible a ~5 FPS, ahorra ~6s de boot)
- **Velocidad de movimiento reducida:** `LERP_SPEED = 0.12`, `PATH_STEP_TICKS = 14`, `MOVE_COOLDOWN = 10`
- **Boot time optimizado:** de 14.6s a 6.4s (-56%) mediante spritesheets + eliminación de outlines redundantes
- **FOV radius por defecto aumentado:** de 10 a 18 tiles

### Eliminado
- **`ConfigMenu` y `SpriteDebug`:** reemplazados por el menu de debug unificado (`DebugMenu`)
- **Constante `CAMERA_ZOOM`:** reemplazada por campo dinamico en `DebugMenu`
- **Assets antiguos:** eliminados sprites de `CelShading/`, `AssetsV1/`, `Ground/`, `Decor/`, y `characters/` (reubicados o reemplazados)

---

## [P3] — 2026-04-05 a 2026-04-07

### Agregado
- **Depth sorting de entidades:** las entidades se intercalan con las filas de tiles usando `entity_depth_row()` basado en posicion visual interpolada, resolviendo el problema de entidades dibujandose sobre muros
- **Sistema de interaccion con NPCs:** presionar E cerca de un NPC abre un dialogo con `GameInput::Interact` y `GameEvent::InteractionStarted`
- **UI de texto con SDL2_ttf:** `TextRenderer` para renderizar texto con fuentes TTF, usado en dialogos y etiquetas de debug
- **Cuadro de dialogo:** muestra nombre del NPC y texto en un panel en la parte inferior de la pantalla, se cierra con E o Escape
- **Entidades definidas en el mapa JSON:** formato extendido con array `"entities"` para spawn de NPCs desde el archivo de mapa
- **Sprites direccionales del jugador:** 8 sprites idle (000-315) y 8x8 sprites de animacion de caminata
- **Animacion de caminata:** sistema de `walk_frame()` con 8 frames por direccion, avance por ticks (`TICKS_PER_ANIM_FRAME`)
- **Facing basado en movimiento:** `grid_dir_to_facing()` mapea (dx, dy) al angulo correcto en proyeccion isometrica
- **Wall cubes:** renderizado de muros como cubos 3D con caras laterales (izquierda/derecha) y cara superior con textura
- **Efectos post-proceso:** dithering (Bayer 4x4 con colores configurables) y Moebius (posterizacion + edge detection)
- **Menu de configuracion (`ConfigMenu`):** ajuste en tiempo real de efectos post-proceso, scope (tiles only/full screen), y zoom
- **Debug de sprites (`SpriteDebug`):** herramienta para ajustar offsets de sprites por direccion en tiempo real
- **Zoom de camara:** constante `CAMERA_ZOOM` aplicada a tiles, entidades, hover y marcadores
- **Hover de tile bajo el mouse:** diamante semi-transparente muestra el tile apuntado
- **`Entity.facing`:** campo para la direccion visual de la entidad (0-315 en pasos de 45)
- **Modulo `post_process`:** funciones `apply_dither()` y `apply_moebius()` con parametros configurables

### Cambiado
- **Reestructuracion de modulos:** codigo separado en `core/` (game_state, entity, input, pathfinding, tilemap, fov), `render/` (renderer, assets, camera, iso, post_process, text), y `ui/` (config_menu, sprite_debug)
- **`main.rs` adelgazado:** de ~200 lineas a ~130 lineas, delegando rendering y UI a modulos
- **Rendering separado en dos fases:** `draw_tiles()` y `draw_entities_and_ui()` para permitir post-proceso entre ambas capas
- **`render_frame()` centralizado:** funcion que orquesta tiles, post-proceso y entidades segun el modo activo

---

## [P2] — 2026-04-05

### Agregado
- **`AssetManager` con lifetimes:** sistema de gestion de texturas con lifetime `'a` vinculado al `TextureCreator` de SDL2 (primera vez usando lifetimes en el proyecto)
- **Carga de PNGs reales:** usando el crate `image` (Rust puro) para decodificar PNGs y crear texturas SDL2 desde surfaces, sin necesidad de SDL2_image
- **Tiles isometricos reales:** sprites de suelo de piedra y dungeon del tileset de Woulette (64x32)
- **Sprites de muros:** texturas `wall_stone_left` y `wall_stone_right` (64x64) con variantes
- **Fallback a placeholders:** si un archivo PNG no existe, se genera una textura placeholder en memoria
- **Darkening con `set_color_mod`:** oscurecimiento de tiles por FOV usando color modulation en vez de rectangulos overlay (que filtraban en areas transparentes)

---

## [P1] — 2026-04-05

### Agregado
- **Arquitectura `GameState`:** la regla dorada `GameInput -> apply_input() -> tick() -> Vec<GameEvent>` que separa logica de rendering
- **Sistema de entidades:** `Entity` con `id`, `EntityKind` (Player/Npc/Enemy), posicion de grilla y visual, pathfinding, y cooldown de movimiento
- **`GameInput` enum:** `MoveDirection` (WASD) y `MoveTo` (click) como mensajes de entrada, preparados para ser la frontera cliente-servidor
- **`GameEvent` enum:** `EntityMoved` y `PathNotFound` como eventos de salida
- **`GameState` pura:** nunca conoce SDL2, texturas, audio ni red; el renderer solo lee `&GameState` (inmutable)
- **Spawn de entidades:** `spawn_entity()` con IDs autoincrementales

### Cambiado
- **Extraccion desde `main.rs`:** la logica de juego se extrajo del game loop monolitico a modulos separados (`game_state.rs`, `entity.rs`, `input.rs`)

---

## [Pre-P1] — Milestones M1-M8 — 2026-04-04 a 2026-04-05

Motor base construido de forma incremental antes de la reestructuracion en fases.

### Agregado
- **M1 — Game loop:** ventana SDL2 con fixed timestep a 60 ticks/segundo
- **M2 — Renderer isometrico:** renderizado de grilla isometrica con scroll de camara. Formulas: `sx = (x-y)*32`, `sy = (x+y)*16` para tiles de 64x32
- **M3 — Tiles con profundidad:** tiles rellenos con diamond fill y depth sorting (fila por fila, atras hacia adelante). Efecto 3D de muros con caras laterales. Contador de FPS en titulo de ventana
- **M4 — Tilemap desde JSON:** carga de mapas desde archivo JSON via serde. `TileKind` enum (Grass/Dirt/Water/Wall) con colores y alturas
- **M5 — Entidad jugador:** movimiento en grilla con WASD, colision contra muros y bordes del mapa
- **M6 — Mapas grandes:** mapa de prueba 200x200 con frustum culling (solo dibuja tiles en pantalla + margen). Movimiento suave con interpolacion visual (lerp)
- **M7 — Pathfinding A*:** algoritmo A* con heuristica Manhattan, 4 direcciones, `BinaryHeap` invertido para min-heap. Click-to-move con marcador de destino
- **M8 — FOV (Field of View):** shadowcasting recursivo de 8 octantes. Brightness por tile con distance falloff (100% en el 50% interno del radio, fade lineal al borde). Ventana redimensionable con `canvas.output_size()` dinamico
