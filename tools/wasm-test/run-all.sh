#!/usr/bin/env bash
# Headless regression suite for the WASM build itself -- run after `./build.sh`
# (or point PICO_R_WASM at a specific artifact) to check the actual shipped
# pico_r.wasm against the same oracle-verified pixel goldens the native
# conformance suite uses, completely independent of the run-cart binary.
set -euo pipefail
cd "$(dirname "$0")"

PROBES=(
  px_lines px_circles px_ovals2 px_drawstate px_font_wide
  px_pal_clip_camera px_print_font px_rect_fillp px_spr_edge px_spr_map px_tline
  # px_p8scii_modes is deliberately excluded: it blanket-pokes 0x5608-0x5fff
  # to fill custom-font data, which as a side effect also clobbers the
  # screen-page redirect register (0x5f55) -- the golden captures raw
  # peek(0x6000+...) memory (unaffected by that redirect), but the actual
  # RENDERED image (what this harness compares) reads from wherever the
  # redirect points, so a naive golden-vs-render comparison mismatches for
  # reasons that are a probe-design quirk, not a pico-r/wasm bug (verified:
  # raw screen memory and the screen-palette table both match the golden
  # exactly; only the final render, which honors the redirected screen
  # page, differs -- and would differ identically on native pico-r and
  # official PICO-8's own display, not just wasm).
)

fail=0
for p in "${PROBES[@]}"; do
  node run-wasm-pixel-probe.mjs "$p" || fail=1
done
exit $fail
