# PICO-R

A pure-WebAssembly PICO-8 emulator in Rust. Plays `.p8` and `.p8.png` carts in the browser with no plugins, no Emscripten, no native toolchain — just `cargo build --target wasm32-unknown-unknown`. PICO-R focuses entirely on the WASM target; PICO-8 compatibility fixes are periodically synced from the sibling Zig project [PICO-Z](https://github.com/mnmlyw/pico-z), but everything else evolves independently.

**▶ [Play in browser](https://mnmlyw.github.io/pico-r/)** — open any `.p8` or `.p8.png` cart directly. No install needed.

To get a cart to try, download [Celeste Classic](https://www.lexaloffle.com/bbs/?tid=2145) or browse the [PICO-8 BBS](https://www.lexaloffle.com/bbs/?cat=7).

> PICO-R is a player/emulator only. To make games, get [PICO-8](https://www.lexaloffle.com/pico-8.php) from Lexaloffle.

## What's in this repo

A complete PICO-8 runtime implemented in safe Rust as a single ~360 KB WebAssembly module:

- **Cart loading** — `.p8` text format and `.p8.png` (manual PNG decoder, steganographic byte extraction, PXA + old compression)
- **Preprocessor** — transforms PICO-8's Lua dialect (short-if/while, compound assignment incl. `^=`, `!=`, peek shortcuts `@`/`%`/`$`, binary literals, bitwise ops `>>`/`<<`/`<<>`/`>>>`/`>><`/`^^`, integer division `\`, `?` print, P8SCII glyph-to-button-ID) to standard Lua 5.2
- **Hand-rolled Lua VM** — purpose-built lexer + parser + tree-walking interpreter. No piccolo or other Lua VM dependency. Tables with metatables, closures, upvalues, varargs, multi-return, full control flow including `goto`/labels and `repeat..until` with proper body-scope condition evaluation
- **Graphics** — pset/line/rect/circ (incl. inverted fill)/oval/spr/sspr/map/tline/print with full P8SCII control codes, pal, fillp, clip, camera
- **Audio** — 4-channel waveform synthesis at 22050 Hz (8 waveforms + custom instruments via child SFX), all 8 SFX effects (slide, vibrato, drop, fade in/out, arpeggio fast/slow), music pattern sequencing with fade
- **Memory** — flat 65536-byte RAM matching PICO-8 layout (sprites, map, SFX, draw state, screen)
- **Input** — keyboard via DOM events; `btn`/`btnp` with PICO-8's repeat-config respect from RAM 0x5F5C/0x5F5D
- **Sandbox** — Lua stdlib subset matching PICO-8 (no `io`, `os`, `debug`, `package`, `require`)
- **Save/Load** — press **P** to save, **L** to load. Same-session is lossless (deep-clones the Lua globals tree, preserves closures stored in tables, audio channels, RNG); cross-session falls back to byte serialization with the closures-in-tables limitation

## Build

Requires `rustup`. The checked-in `rust-toolchain.toml` pins stable Rust, `rustfmt`, `clippy`, and the wasm32 target.

```bash
# One-time toolchain setup (Homebrew Rust does NOT include the wasm32 target):
brew install rustup-init && rustup-init -y

# Build the WASM module and copy it into web/:
./build.sh

# Serve the web/ directory locally:
python3 -m http.server -d web 8765
# Open http://localhost:8765
```

`build.sh` will also run `wasm-opt -Oz` if you have it installed (Homebrew: `brew install binaryen`).

## Architecture

| Module | Lines | Purpose |
|---|---|---|
| `memory.rs` | 214 | 65536-byte RAM/ROM, screen/sprite/map indexing |
| `palette.rs` | 22 | 32-color ARGB palette (16 standard + 16 extended) |
| `cart.rs` | 609 | `.p8` section parser + `.p8.png` PNG decoder + PXA decompression |
| `preprocessor.rs` | 1367 | Line-by-line PICO-8 → Lua 5.2 transform |
| `gfx.rs` + `gfx_font.rs` | 785 | Drawing primitives, sprite blitting, P8SCII font rendering |
| `audio.rs` | 656 | 4-channel synthesis, returned to JS as f32 samples |
| `input.rs` | 74 | btn state + held-frame counters for btnp repeat |
| `state.rs` | 53 | Top-level `PicoState` aggregating engine state |
| `pico_lua/lex.rs` | 280 | Lua 5.2 tokenizer |
| `pico_lua/ast.rs` | 60 | AST node types |
| `pico_lua/parse.rs` | 290 | Recursive-descent parser |
| `pico_lua/value.rs` | 220 | Value enum, Table with array+hash, deep-clone helpers |
| `pico_lua/interp.rs` | 470 | Tree-walking evaluator |
| `pico_lua/api.rs` | 870 | ~80 PICO-8 API functions registered as native callbacks |
| `pico_lua/serialize.rs` | 165 | Tagged binary save/load for Lua globals |
| `pico_lua/mod.rs` | 130 | LuaEngine trait impl, env-fallback shim |
| `lib.rs` | 285 | WASM exports, frame loop, in-memory snapshot |
| `web/index.html` | — | Drop-in cart loader, keyboard input, audio resampler, save UI |

The output is a single self-contained `.wasm` with **zero JS imports** — Rust's `dlmalloc` is the global allocator, all I/O goes through explicit exports.

## WASM exports

The WASM module exports:

| Export | Purpose |
|---|---|
| `web_alloc(len)` / `web_free(ptr, len)` | JS allocates a buffer in WASM memory |
| `web_init(ptr, len)` | Load cart from bytes; returns 0 on success |
| `web_update()` | One frame: input → update → draw → render |
| `web_get_pixel_buffer()` | Pointer to the 128×128 ARGB pixel buffer |
| `web_set_buttons(player, bits)` | Set button state for player 0 or 1 |
| `web_set_mouse(x, y, btns, wheel)` | Mouse state |
| `web_generate_audio(n)` | Returns a pointer to `n` f32 audio samples at 22050 Hz |
| `web_get_fps()` | 30 or 60 |
| `web_save_state()` / `web_get_save_ptr()` / `web_free_save()` / `web_load_state(ptr, len)` | Quick save/load |
| `web_has_error()` | 1 if the Lua VM hit an error |

## Controls

| Key | Action |
|---|---|
| Arrow keys | D-pad |
| Z / C / N | O button |
| X / V / M | X button |
| S F E D | Player 2 D-pad |
| Tab / Q / W | Player 2 buttons |
| P | Save state (lossless within session) |
| L | Load state |

## Tooling

There's a host-side cart runner for fast iteration without browser round-trips:

```bash
cargo run --bin run-cart --release -- /path/to/cart.p8.png 200
```

Loads a cart, simulates pressing X for the first 8 frames (to advance past the title), runs N frames, and prints the first error with line info:

```
UPDATE ERROR (frame 81): cart:2032 in _update: compare nil with number
```

This is how the `repeat..until`-scope and broken-`all()`-iterator bugs were diagnosed during development.

A sibling `cargo run --bin dump-pp --release -- /path/to/cart.p8.png [output.lua]` writes the preprocessed Lua source so you can `sed -n '2030,2040p' output.lua` and see the actual code at the failing line.

## Compatibility

Tested with [Celeste Classic 2](https://www.lexaloffle.com/bbs/?tid=41339) and other community carts.

**Known limitations:**
- **No coroutines.** Carts using `flip()` / `cocreate` / `coresume` / `yield` for their own frame loop won't work. Carts using the standard `_init`/`_update`/`_draw` lifecycle (the dominant pattern) run fine.
- **Numbers are f64, not bit-exact 16:16 fixed-point.** Pure-arithmetic carts won't notice; carts that rely on overflow tricks at exactly ±32768 or specific `tostr(n, 0x1)` hex formats may misbehave at edges.
- **`string.find`/`match`/`gmatch`/`gsub`** not implemented — rare in PICO-8 carts; `split`/`sub`/`chr`/`ord` cover most cases.
- **Cross-session save/load drops closures stored in tables** (the same-session in-memory snapshot path preserves them).

## Credits

- **[PICO-Z](https://github.com/mnmlyw/pico-z)** — sibling Zig PICO-8 emulator. PICO-R borrows from its PICO-8 compatibility layer; the WASM-only architecture, hand-rolled Lua VM, and rest of the engine are independent.
- **[PICO-8](https://www.lexaloffle.com/pico-8.php)** by [Joseph White / Lexaloffle](https://www.lexaloffle.com/) — the original. PICO-R is not affiliated.
- **[z8lua](https://github.com/samhocevar/z8lua)** by Sam Hocevar (WTFPL) — bit-exact `sin`/`atan2` lookup tables.

## License

MIT.
