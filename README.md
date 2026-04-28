# PICO-R

Pure-WASM Rust port of [PICO-Z](https://github.com/mnmlyw/pico-z) — a PICO-8
emulator. No native build needed: `cargo build --target wasm32-unknown-unknown`
and serve `web/`.

## Status

| Module | Status |
|---|---|
| Memory layout (RAM/ROM, screen, sprite, map) | ✅ ported |
| Cart loading (`.p8` text + `.p8.png` PNG steganography, PXA + old compression) | ✅ ported |
| PICO-8 → Lua 5.2 preprocessor (short-if, compound assigns, peek shortcuts, bitwise, etc.) | ✅ ported |
| Graphics (cls, line, rect, circ, oval, spr, sspr, map, print with P8SCII) | ✅ ported |
| Audio synthesis (4-channel, 8 waveforms, effects, music) | ✅ ported |
| Input (button state, btnp repeat) | ✅ ported |
| Trig tables (z8lua sin/atan2 lookup) | ✅ ported |
| WASM exports + HTML host | ✅ wired |
| **Lua VM** | ⚠️ **stub only** — see below |

## Wiring a real Lua VM

`src/lua_engine.rs` exposes a `LuaEngine` trait. The default `StubEngine`
preprocesses cart source but does not execute it — instead it draws a
"PICO-R / no Lua VM wired" placeholder so you can confirm the WASM/HTML
pipeline works end-to-end.

To run real carts, implement `LuaEngine` against any pure-Rust Lua VM. The
two viable options:

1. **[piccolo](https://crates.io/crates/piccolo)** — pure Rust, full GC.
   Skeleton dependency is already in `Cargo.toml` behind `--features piccolo`.
   You'll need to register the PICO-8 API (cls, pset, spr, etc.) using
   `Callback::from_fn` and store `&mut PicoState` via `UserData` in the
   Lua registry.
2. **Hand-rolled subset interpreter** — after the preprocessor, the Lua
   subset PICO-8 needs is small (no `require`, `io`, `os`, etc.). A
   tree-walker in ~1500 lines is feasible if piccolo's GC-arena lifetimes
   are too restrictive for your taste.

The trait surface is minimal:

```rust
pub trait LuaEngine {
    fn load_cart(&mut self, state: &mut PicoState, cart: &Cart) -> Result<(), String>;
    fn call_init(&mut self, state: &mut PicoState);
    fn call_update(&mut self, state: &mut PicoState);
    fn call_draw(&mut self, state: &mut PicoState);
    fn use_60fps(&self) -> bool;
    fn had_error(&self) -> bool;
    fn error_message(&self) -> &str;
}
```

The PICO-8 API your Lua bindings need to call on `PicoState`:
graphics functions in `crate::gfx::*`, audio in `state.audio.play_sfx/play_music`,
RNG in `state.rng_state`, input checks via `state.input.btn(...)`.

## Build

```bash
# One-time toolchain setup (Homebrew Rust does not include the wasm32 target):
brew install rustup-init && rustup-init -y
rustup target add wasm32-unknown-unknown

# Build the WASM module and copy into web/:
./build.sh

# Serve the web/ directory locally:
python3 -m http.server -d web 8000
# Open http://localhost:8000
```

## Architecture

The same shape as PICO-Z, modulo the language change:

- `memory.rs` — flat 65536-byte RAM matching PICO-8 layout
- `palette.rs` — 32-color palette (16 standard + 16 extended)
- `cart.rs` — `.p8` section parser + `.p8.png` PNG decoder + PXA decompression
- `preprocessor.rs` — line-by-line PICO-8 → Lua 5.2 transform
- `gfx.rs` + `gfx_font.rs` — drawing primitives, sprite blitting, font
- `audio.rs` — 4-channel synthesis at 22050 Hz, returned to JS as f32 samples
- `input.rs` — btn state + held-frame counters for btnp repeat
- `state.rs` — top-level `PicoState` aggregating all of the above
- `lua_engine.rs` — `LuaEngine` trait + `StubEngine`
- `lib.rs` — WASM exports (`web_init`, `web_update`, `web_alloc`, …)

## Exports

The WASM module exports the same surface as PICO-Z's web build, so
`web/index.html` can drive it the same way:

| Export | Purpose |
|---|---|
| `web_alloc(len)` / `web_free(ptr, len)` | JS allocates a buffer in WASM memory |
| `web_init(ptr, len)` | Load cart from bytes; returns 0 on success |
| `web_update()` | One frame: input → update → draw → render |
| `web_get_pixel_buffer()` | Pointer to the 128×128 ARGB pixel buffer |
| `web_set_buttons(player, bits)` | Set button state for player 0 or 1 |
| `web_set_mouse(x, y, btns, wheel)` | Mouse state |
| `web_generate_audio(n_samples)` | Returns a pointer to n f32 audio samples |
| `web_get_fps()` | 30 or 60 |
| `web_save_state()` / `web_get_save_ptr()` / `web_free_save()` / `web_load_state(ptr, len)` | Quick save/load |

## Credits

This is a Rust rewrite of [PICO-Z](https://github.com/mnmlyw/pico-z) by mnmlyw,
which was itself an independent reimplementation of the PICO-8 runtime.
PICO-8 itself is by Joseph White / Lexaloffle. Trig tables are from
[z8lua](https://github.com/samhocevar/z8lua) by Sam Hocevar (WTFPL).
