#!/usr/bin/env bash
# Build pico-r to WASM and copy the artifact into web/.
#
# Requires the wasm32-unknown-unknown target installed:
#   rustup target add wasm32-unknown-unknown

set -euo pipefail

cd "$(dirname "$0")"

cargo build --locked --release --target wasm32-unknown-unknown

OUT_WASM="target/wasm32-unknown-unknown/release/pico_r.wasm"
DEST="web/pico-r.wasm"

if [ ! -f "$OUT_WASM" ]; then
  echo "Build artifact not found: $OUT_WASM" >&2
  exit 1
fi

cp "$OUT_WASM" "$DEST"
echo "Wrote $DEST ($(wc -c < "$DEST") bytes)"

# Optional: shrink with wasm-opt if available.
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Oz -o "$DEST" "$DEST"
  echo "After wasm-opt: $(wc -c < "$DEST") bytes"
fi
