# CLAUDE.md — Aprendizaje de Rust + Videojuego Isométrico

## Contexto del proyecto

Estoy aprendiendo Rust viniendo de JavaScript/TypeScript (Node.js). Entiendo programación — tipos, funciones, estructuras de datos, async, etc. — y ya completé los fundamentos de Rust (Fase 1).

Ahora estoy en la **Fase 2 — construir un videojuego isométrico** estilo Fallout 1.

---

## Mi perfil como desarrollador

- Lenguajes dominados: JavaScript, TypeScript, Node.js
- Conceptos que ya manejo de JS: tipos, interfaces, generics (TS), async/await, closures, módulos, estructuras de datos
- IDE: Zed (con rust-analyzer integrado)
- Entorno: Windows 10, Rust 1.94.1, Claude Code como asistente
- Quiero entender cada línea que escribo — no solo que funcione

---

## Fase 1 — Completada

Conceptos de Rust que ya aprendí y entiendo:

| Concepto | Capítulo Rust Book | Estado |
|----------|-------------------|--------|
| Variables, mutabilidad, `let` vs `const` | 3 | Completado |
| Tipos de datos (`i32`, `u8`, `f64`, `bool`, `&str`, `String`) | 3 | Completado |
| Funciones, retorno implícito, macros vs funciones | 3 | Completado |
| Control de flujo (`if` como expresión, `match`, `for`, `loop`, `while`) | 3 | Completado |
| Ownership, move, Copy types | 4 | Completado |
| Borrowing (`&`, `&mut`), reglas de borrowing | 4 | Completado |
| Slices (`&str`, `&[T]`) | 4 | Completado |
| Structs, métodos, `impl`, funciones asociadas | 5 | Completado |
| Enums con datos, pattern matching exhaustivo | 6 | Completado |
| `Option<T>` (`Some`/`None`) | 6 | Completado |
| Colecciones: `Vec<T>`, `String`, `HashMap<K, V>` | 8 | Completado |
| Error handling: `Result<T, E>`, `panic!`, operador `?` | 9 | Completado |
| Traits, `impl Trait for Struct`, `derive`, trait bounds | 10 | Completado |
| Generics `<T>` | 10 | Completado |
| Closures (`\|x\| x + 1`), `move` | 13 | Completado |
| Iteradores (`.iter()`, `.map()`, `.filter()`, `.collect()`, etc.) | 13 | Completado |
| Módulos (`mod`, `use`, `pub`, carpetas con `mod.rs`, `crate::`) | 7 | Completado |

### Conceptos de Rust que AÚN NO aprendí

Estos conceptos aparecerán en la Fase 2. **Cuando sea necesario usar alguno, explicarlo antes de usarlo, comparando con JS/TS.**

| Concepto | Cuándo va a aparecer |
|----------|---------------------|
| **Lifetimes (`'a`)** | Cuando una struct guarde referencias en vez de owned data |
| **Box, Rc, Arc** (smart pointers) | Si necesitamos polimorfismo dinámico o datos compartidos |
| **trait objects (`dyn Trait`)** | Si necesitamos distintos tipos en una misma colección |
| **async/await** | Probablemente no en Fase 2, el game loop es sincrónico |
| **unsafe** | Si SDL2 lo requiere en algún binding |
| **Lifetimes en structs** | Cuando una struct necesite guardar un `&str` en vez de `String` |
| **Error types personalizados** | Cuando crezca el manejo de errores del juego |
| **Closures como parámetros (`Fn`, `FnMut`, `FnOnce`)** | Cuando pasemos callbacks a sistemas del juego |

---

## Reglas de interacción

### Explicaciones
- **Siempre comparar con JS/TS** cuando introduzcas un concepto nuevo.
- **Explicar el "por qué"** detrás de las decisiones de diseño, no solo el "cómo".
- Si un concepto tiene un capítulo en The Rust Book, mencionarlo.
- **Si un milestone requiere un concepto nuevo** (de la tabla de arriba), explicarlo primero con un ejemplo pequeño antes de usarlo en el juego.

### Código
- **No escribir código que no pueda explicar.** Estar preparado para explicar cada línea si pregunto.
- **Preferir código explícito sobre código idiomático** hasta que yo conozca ambas formas. Mostrar la explícita primero, la idiomática después.
- **Compilar siempre en la cabeza antes de sugerir.** No sugerir código que el borrow checker va a rechazar sin advertirlo.
- Usar `cargo check` y `cargo clippy` como herramientas de feedback.

### Ritmo
- Avanzar un concepto a la vez.
- Cuando algo no compile, **ayudarme a leer el error del compilador** en lugar de darme la solución directa.
- Si el código usa un concepto que no vi, **parar y explicar antes de seguir**.

---

## Fase 2 — Videojuego isométrico (EN CURSO)

### Concepto
RPG/exploración con vista isométrica. Estética similar a Fallout 1: gráficos pre-renderizados en 2D isométrico, tile-based, con niebla de guerra y pathfinding.

### Stack técnico
- **Lenguaje:** Rust
- **Librería gráfica:** SDL2 (`sdl2` crate con feature `image`)
- **Sin motor de juego.** Todo el rendering, game loop, y sistemas son código propio.

### Milestones

| Milestone | Objetivo | Estado |
|-----------|----------|--------|
| **M1** | Ventana + game loop con fixed timestep | Pendiente |
| **M2** | Tile renderer isométrico (proyección iso, grid, camera scroll) | Pendiente |
| **M3** | Sprites + depth sorting correcto | Pendiente |
| **M4** | Tilemap desde archivo (RON o JSON) | Pendiente |
| **M5** | Entidad player + movimiento (click-to-move o WASD) | Pendiente |
| **M6** | Cámara que sigue al player + mapa grande | Pendiente |
| **M7** | A* pathfinding sobre grid isométrico | Pendiente |
| **M8** | FOV / Shadowcasting (niebla de guerra, línea de visión) | Pendiente |

### Conceptos técnicos a dominar en Fase 2
- Game loop determinístico: fixed timestep, separación update/render
- Proyección isométrica: conversión screen ↔ world coords
- Depth sorting: z-ordering de tiles y entidades en iso
- Sprite sheet blitting con SDL2
- A* sobre un grid con obstáculos
- Shadowcasting para FOV (algoritmo de Björn Bergström o similar)

---

## Recurso de referencia

**The Rust Book:** https://doc.rust-lang.org/stable/book/

---

## Vocabulario JS → Rust (referencia rápida)

| JavaScript/TypeScript | Rust |
|-----------------------|------|
| `let x = 5` (mutable por defecto) | `let mut x = 5` (inmutable por defecto) |
| `const x = 5` | `let x = 5` |
| `undefined` / `null` | `Option<T>` (`None` / `Some(value)`) |
| `throw new Error(...)` | `Err(...)` con `Result<T, E>` |
| `try/catch` | `match result { Ok(v) => ..., Err(e) => ... }` o `?` |
| `interface Foo { ... }` | `trait Foo { ... }` |
| `class Foo implements Bar` | `struct Foo; impl Bar for Foo { ... }` |
| `Array<T>` | `Vec<T>` |
| `Map<K, V>` | `HashMap<K, V>` |
| Closures `(x) => x + 1` | Closures `\|x\| x + 1` |
| Módulos ES (`import/export`) | `mod`, `use`, `pub` |
| `typeof` / duck typing | Traits + generics en tiempo de compilación |
| Garbage collector | Ownership + borrow checker (tiempo de compilación) |
| `switch` (con fall-through) | `match` (exhaustivo, sin fall-through) |
| `?.` optional chaining | `if let Some(x) = ...` o `?` operator |
| `npm` / `package.json` | `cargo` / `Cargo.toml` |
| `node index.js` | `cargo run` |
| `eslint` | `cargo clippy` |

---

## Notas adicionales

- Responder siempre en español.
- Si hay varias formas de hacer algo, mostrar la más explícita primero y la más idiomática después, explicando la diferencia.
- Cuando el compilador de Rust rechace algo, ayudar a leer el mensaje de error antes de dar la solución.
- El objetivo final es un juego funcional donde entienda cada línea del código, no solo que funcione.
