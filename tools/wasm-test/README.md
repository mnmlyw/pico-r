# WASM build regression tests

Headless, Node-based tests that exercise the actual compiled `pico_r.wasm`
artifact through the same exported interface `web/index.html` uses
(`web_alloc`/`web_init`/`web_update`/`web_get_pixel_buffer`, ...) — no
browser required. This is a separate check from `cargo test`'s conformance
suite, which only runs the native `run-cart` binary; the wasm32 target is
a genuinely different compile (different codegen, no threads, `panic=abort`)
and could have its own bugs the native suite can't see.

## Running

```bash
./build.sh                    # produces web/pico-r.wasm
node tools/wasm-test/run-wasm-pixel-probe.mjs <probe-name>   # single probe
tools/wasm-test/run-all.sh    # the full suite
```

## How it works

Most of `tests/conformance/probes/*.p8` verify via `printh()` text output,
which is a no-op on wasm32 (there's no stdout in a browser) — those can't be
replayed this way. But the **pixel-conformance probes** (`px_*.p8`) instead
dump the full screen as a `peek(0x6000+...)` hex golden, which this harness
can independently reconstruct into the same ARGB image `gfx::render_to_argb`
produces (applying the live screen-palette remap table, read back via a
small `web_debug_peek(addr)` debug export) and compare pixel-for-pixel
against the wasm build's actual `web_get_pixel_buffer()` output.

`px_p8scii_modes` is deliberately excluded from `run-all.sh` — see the
comment there. It's a probe-design quirk (the probe's own fill loop
clobbers the screen-page redirect register as a side effect), not a
pico-r/wasm bug, and would mismatch this harness's comparison method on
*any* build, native included.
