# PICO-R

A pure-WebAssembly PICO-8 emulator in Rust. Plays `.p8` and `.p8.png` carts in the browser with no plugins, no Emscripten, no native toolchain — just `cargo build --target wasm32-unknown-unknown`. PICO-R focuses entirely on the WASM target; PICO-8 compatibility fixes are periodically synced from the sibling Zig project [PICO-Z](https://github.com/mnmlyw/pico-z), but everything else evolves independently.

**▶ [Play in browser](https://mnmlyw.github.io/pico-r/)** — open any `.p8` or `.p8.png` cart directly. No install needed.

To get a cart to try, download [Celeste Classic](https://www.lexaloffle.com/bbs/?tid=2145) or browse the [PICO-8 BBS](https://www.lexaloffle.com/bbs/?cat=7).

> PICO-R is a player/emulator only. To make games, get [PICO-8](https://www.lexaloffle.com/pico-8.php) from Lexaloffle.

## What's in this repo

A complete PICO-8 runtime implemented in safe Rust as a single ~360 KB WebAssembly module:

- **Cart loading** — `.p8` text format and `.p8.png` (manual PNG decoder, steganographic byte extraction, PXA + old compression); the PNG ROM image is loaded byte-verbatim into RAM (verified against the official binary's own `reload()` of the same file)
- **Preprocessor** — transforms PICO-8's Lua dialect (short-if/while, compound assignment incl. `^=`, `!=`, peek shortcuts `@`/`%`/`$`, binary literals, bitwise ops `>>`/`<<`/`<<>`/`>>>`/`>><`/`^^`, integer division `\`, `?` print, P8SCII glyph identifiers and glyph-to-button-ID) to standard Lua 5.2, operating on raw bytes end-to-end so high-byte glyphs survive as identifier characters
- **Hand-rolled Lua VM** — purpose-built lexer + parser + tree-walking interpreter. No piccolo or other Lua VM dependency. Tables with metatables and insertion-ordered storage, closures, upvalues, varargs, multi-return, real `local _ENV` scoping, `goto`/labels, `repeat..until` with proper body-scope condition evaluation, and coroutines (native builds; the wasm build has no threads and compiles them out)
- **Bit-exact numbers** — true 16.16 fixed-point quantization on literals and arithmetic, PICO-8's actual `rnd()`/`srand()` PRNG algorithm, and z8lua's `sin`/`cos`/`atan2` lookup tables, all locked to the official binary's output
- **Graphics** — pset/line/rect/circ (incl. inverted fill)/oval/spr/sspr/map/print with full P8SCII control codes, pal, fillp, clip, camera (`tline` is currently a parse-but-draw-nothing stub)
- **Audio** — 4-channel waveform synthesis at 22050 Hz (8 waveforms + custom instruments via child SFX), all 8 SFX effects (slide, vibrato, drop, fade in/out, arpeggio fast/slow), music pattern sequencing with fade
- **Memory** — flat 65536-byte RAM matching PICO-8 layout (sprites, map, SFX, draw state, screen, big-map region at 0x8000+ incl. the 0x5F56/0x5F57 custom map registers)
- **Multi-cart** — real `load()` cart switching with breadcrumb return via `extcmd("breadcrumb")`, external-file `reload()`, and high-RAM (0x8000+) persistence across switches — enough to run BBS games that stage data through a companion cart
- **Input** — keyboard via DOM events; `btn`/`btnp` with PICO-8's repeat-config respect from RAM 0x5F5C/0x5F5D
- **Sandbox** — Lua stdlib subset matching PICO-8 (no `io`, `os`, `debug`, `package`, `require` — and, matching the real console, no `string` library either; string indexing `s[i]`, `split`, `sub`, `chr`, `ord` are the PICO-8 way). The shell commands (`help`, `ls`, `save`, `run`, ...) exist as callable globals just like on the official console
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
| `memory.rs` | 278 | 65536-byte RAM/ROM, screen/sprite/map indexing |
| `palette.rs` | 21 | 32-color ARGB palette (16 standard + 16 extended) |
| `cart.rs` | 598 | `.p8` section parser + `.p8.png` PNG decoder + PXA decompression |
| `preprocessor.rs` | 1972 | Byte-level PICO-8 → Lua 5.2 transform |
| `gfx.rs` + `gfx_font.rs` | 1015 | Drawing primitives, sprite blitting, P8SCII font rendering |
| `audio.rs` | 708 | 4-channel synthesis, returned to JS as f32 samples |
| `input.rs` | 88 | btn state + held-frame counters for btnp repeat |
| `state.rs` | 110 | Top-level `PicoState` aggregating engine state |
| `pico_lua/lex.rs` | 687 | Lua 5.2 tokenizer (raw bytes; high-byte glyphs are identifier chars) |
| `pico_lua/ast.rs` | 85 | AST node types |
| `pico_lua/parse.rs` | 564 | Recursive-descent parser |
| `pico_lua/value.rs` | 577 | Value enum, insertion-ordered Table, deep-clone helpers |
| `pico_lua/interp.rs` | 1002 | Tree-walking evaluator (`_ENV` scoping, metamethods, CPU-budget accounting) |
| `pico_lua/api.rs` | 2364 | ~100 PICO-8 API functions registered as native callbacks |
| `pico_lua/coroutine.rs` | 198 | Coroutines via baton-passing OS threads (native only) |
| `pico_lua/serialize.rs` | 269 | Tagged binary save/load for Lua globals |
| `pico_lua/mod.rs` | 292 | LuaEngine trait impl |
| `lib.rs` | 461 | WASM exports, frame loop, in-memory snapshot |
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

This is how the `repeat..until`-scope and broken-`all()`-iterator bugs were diagnosed during development. If a cart hangs instead of erroring, `PICOR_TRACE=1` logs the executing line/depth periodically, and `PICOR_LUA_OVERRIDE=/path/to/alt.lua` runs alternate Lua source against the cart's real ROM data (useful for instrumenting a cart's own code with `printh` probes).

Sibling dump tools write a cart's Lua source to a file: `dump-pp` emits the preprocessed Lua 5.2 (so you can see the actual code at a failing line) and `dump-raw` the original PICO-8-dialect source:

```bash
cargo run --bin dump-pp --release -- /path/to/cart.p8.png output.lua
cargo run --bin dump-raw --release -- /path/to/cart.p8.png output.lua
```

## Conformance testing

Engine behavior is locked to the **official PICO-8 binary as an empirical oracle**, not to documentation or folklore. `tests/conformance/` holds 70+ probe carts (`probes/*.p8`), each exercising one behavior — fixed-point overflow, `pairs()` ordering, string indexing, P8SCII escapes, trig table boundaries, `next()` over object keys, ... — with golden outputs (`golden/*.txt`) captured by running the probe on the real console:

```bash
tools/oracle.sh tests/conformance/probes/<name>.p8   # (re)generate a golden on a machine with PICO-8
cargo test --test conformance                        # every probe must match its golden byte-for-byte
```

Goldens are checked in, so CI and contributors never need the PICO-8 binary. `tests/conformance/LEDGER.md` is the running log of every divergence found and fixed, with the evidence for each.

## Compatibility

Measured against a corpus of **188 BBS carts**: **187 run clean; the remaining 1 fails identically on the official binary** (it's listed in `tests/conformance/broken_on_official.txt`, where an error exit *is* the conformant outcome). That includes heavyweights like embedded LISP VMs, LZW self-decompressors, multi-cart loaders, and coroutine-driven frame loops.

**Known limitations:**
- **`tline()` draws nothing** — parsed and callable, but its raster algorithm hasn't been oracle-verified yet, so it's a deliberate no-op rather than a wrong guess.
- **No coroutines on the WASM build** (no threads there; native builds have them via OS threads).
- **Frame pacing** — `_set_fps()` and `menuitem()` are accepted but inert; the host drives the frame rate.
- **Cross-session save/load drops closures stored in tables** (the same-session in-memory snapshot path preserves them).

## Credits

- **[PICO-Z](https://github.com/mnmlyw/pico-z)** — sibling Zig PICO-8 emulator. PICO-R borrows from its PICO-8 compatibility layer; the WASM-only architecture, hand-rolled Lua VM, and rest of the engine are independent.
- **[PICO-8](https://www.lexaloffle.com/pico-8.php)** by [Joseph White / Lexaloffle](https://www.lexaloffle.com/) — the original. PICO-R is not affiliated.
- **[z8lua](https://github.com/samhocevar/z8lua)** by Sam Hocevar (WTFPL) — bit-exact `sin`/`atan2` lookup tables.

## License

MIT.
