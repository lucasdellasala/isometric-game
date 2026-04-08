# CHANGELOG

Registro de cambios del RPG isometrico multijugador estilo D&D, construido desde cero en Rust con SDL2.

---

## [Unreleased] — En desarrollo

### Agregado
- **Spritesheets de NPCs con variantes:** 9 variantes visuales (african, caucasian, latino x black/brown/cream) con spritesheets de 1024x256 (8 direcciones por hoja)
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
  - `assets/sprites/npc/` para spritesheets de NPCs
  - `assets/sprites/enemy/` para enemigos
  - `assets/sprites/decorations/` para decoraciones
  - `assets/tiles/forest/`, `assets/tiles/water/`, `assets/tiles/terrain/` para tiles de terreno
  - `assets/maps/` para archivos JSON de mapas
  - `assets/fonts/` para fuentes TTF
- **Centrado de camara:** la camara ahora centra verticalmente en `screen_h / 2` en vez de `screen_h / 4`
- **Spawn del jugador al centro del mapa:** en vez de (0,0), ahora aparece en `(cols/2, rows/2)`
- **Tile rendering normalizado:** `draw_tile()` siempre dibuja a `TILE_WIDTH x TILE_HEIGHT` independientemente del tamano real del sprite
- **Frustum culling ampliado:** margen de culling duplicado a `TILE_WIDTH * 2` para evitar pop-in con zoom
- **Entity rendering con spritesheet src_rect:** `entity_texture_info()` retorna `(key, Option<Rect>)` para soportar recorte de spritesheets de NPCs
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
