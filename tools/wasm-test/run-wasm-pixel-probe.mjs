#!/usr/bin/env node
// Headless conformance harness for the WASM build itself -- exercises
// pico-r.wasm through the SAME exported interface web/index.html uses
// (web_alloc/web_init/web_get_pixel_buffer), completely independent of
// the native run-cart binary. Verifies pixel-conformance probes (whose
// goldens are raw 4bpp screen dumps captured from official PICO-8, not
// printh text -- printh is a no-op on wasm32, so RESULT/DONE-style
// probes can't be replayed this way) by rendering the golden's screen
// bytes through the same ARGB palette table pico-r's renderer uses and
// comparing pixel-for-pixel against the wasm build's actual output.
//
// Usage: node run-wasm-pixel-probe.mjs <probe-name> [--frames N]
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(__dirname, '..', '..');

const PALETTE = [
  0xFF000000, 0xFF1D2B53, 0xFF7E2553, 0xFF008751, 0xFFAB5236, 0xFF5F574F, 0xFFC2C3C7, 0xFFFFF1E8,
  0xFFFF004D, 0xFFFFA300, 0xFFFFEC27, 0xFF00E436, 0xFF29ADFF, 0xFF83769C, 0xFFFF77A8, 0xFFFFCCAA,
  0xFF291814, 0xFF111D35, 0xFF422136, 0xFF125359, 0xFF742F29, 0xFF49333B, 0xFFA28879, 0xFFF3EF7D,
  0xFFBE1250, 0xFFFF6C24, 0xFFA8E72E, 0xFF00B543, 0xFF065AB5, 0xFF754665, 0xFFFF6E59, 0xFFFF9D81,
];

function parseGoldenScreenDump(text, screenPal) {
  // Expects lines "RESULT rNNN <128 hex chars>" (64 bytes/row, 128 rows).
  // screenPal[i] is ADDR_SCREEN_PAL's live remap for palette index i
  // (read from the SAME wasm instance after running the probe) -- the
  // golden's raw bytes are palette INDICES straight from screen memory
  // (peek(0x6000+...)), matching what pico-r's own screen memory holds
  // before the display-time remap gfx::render_to_argb applies via
  // get_screen_pal. Skipping this remap would flag a probe that legitimately
  // pokes the screen-palette registers (e.g. as a side effect of filling a
  // nearby memory region) as a false mismatch.
  const rows = new Array(128).fill(null);
  for (const line of text.split('\n')) {
    const m = line.match(/^RESULT r(\d+) ([0-9a-f]{128})$/);
    if (m) rows[parseInt(m[1], 10)] = m[2];
  }
  if (rows.some((r) => r === null)) {
    throw new Error('golden is missing one or more row lines -- not a screen-dump probe');
  }
  // Each hex row is 64 bytes; each byte packs 2 pixels (low nibble = even x, high nibble = odd x).
  const argb = new Uint32Array(128 * 128);
  for (let y = 0; y < 128; y++) {
    const row = rows[y];
    for (let b = 0; b < 64; b++) {
      const byte = parseInt(row.substr(b * 2, 2), 16);
      const lo = byte & 0x0f;
      const hi = (byte >> 4) & 0x0f;
      argb[y * 128 + b * 2] = PALETTE[screenPal[lo]];
      argb[y * 128 + b * 2 + 1] = PALETTE[screenPal[hi]];
    }
  }
  return argb;
}

async function main() {
  const probeName = process.argv[2];
  if (!probeName) {
    console.error('usage: run-wasm-pixel-probe.mjs <probe-name> [--frames N]');
    process.exit(2);
  }
  const framesIdx = process.argv.indexOf('--frames');
  // Default 1, not 0: web_update() is what triggers gfx::render_to_argb --
  // the pixel buffer stays at its untouched init value (opaque black)
  // until at least one frame runs, even for probes that draw entirely
  // during _init/top-level (none of these define _update/_draw, so one
  // web_update() call is a safe no-op dispatch that still renders).
  const frames = framesIdx >= 0 ? parseInt(process.argv[framesIdx + 1], 10) : 1;

  const wasmPath = join(repoRoot, 'web', 'pico-r.wasm');
  const probePath = join(repoRoot, 'tests', 'conformance', 'probes', `${probeName}.p8`);
  const goldenPath = join(repoRoot, 'tests', 'conformance', 'golden', `${probeName}.txt`);

  const wasmBytes = readFileSync(wasmPath);
  const { instance } = await WebAssembly.instantiate(wasmBytes, {});
  const wasm = instance.exports;
  const mem = () => wasm.memory.buffer;

  const cartBytes = readFileSync(probePath);
  const ptr = wasm.web_alloc(cartBytes.length);
  if (!ptr) throw new Error('web_alloc failed');
  new Uint8Array(mem(), ptr, cartBytes.length).set(cartBytes);
  const err = wasm.web_init(ptr, cartBytes.length);
  wasm.web_free(ptr, cartBytes.length);
  if (err !== 0) throw new Error(`web_init returned ${err} (cart failed to load)`);

  for (let f = 0; f < frames; f++) wasm.web_update();

  const bufPtr = wasm.web_get_pixel_buffer();
  const actual = new Uint32Array(mem(), bufPtr, 128 * 128).slice();

  // ADDR_SCREEN_PAL = 0x5F10, 16 bytes, low nibble is the live remap
  // target for that palette index (see memory.rs/gfx.rs get_screen_pal).
  const screenPal = [];
  for (let i = 0; i < 16; i++) screenPal.push(wasm.web_debug_peek(0x5f10 + i) & 0x0f);

  const goldenText = readFileSync(goldenPath, 'utf8');
  const expected = parseGoldenScreenDump(goldenText, screenPal);

  let mismatches = 0;
  const sample = [];
  for (let i = 0; i < 128 * 128; i++) {
    if (actual[i] !== expected[i]) {
      mismatches++;
      if (sample.length < 8) {
        const x = i % 128, y = Math.floor(i / 128);
        sample.push(`(${x},${y}) expected=0x${expected[i].toString(16)} actual=0x${actual[i].toString(16)}`);
      }
    }
  }

  if (mismatches === 0) {
    console.log(`PASS  ${probeName}: 16384/16384 pixels match (wasm build == oracle-verified golden)`);
    process.exit(0);
  } else {
    console.log(`FAIL  ${probeName}: ${mismatches}/16384 pixels differ`);
    for (const s of sample) console.log('  ' + s);
    process.exit(1);
  }
}

main().catch((e) => { console.error(e); process.exit(1); });
