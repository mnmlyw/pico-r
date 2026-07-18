#!/bin/sh
# Run a probe cart through the official PICO-8 binary and write its golden
# output. This is the ONLY place official PICO-8 is invoked — the resulting
# golden files are checked in, so CI and fix agents never need the binary.
#
# Usage: tools/oracle.sh tests/conformance/probes/<name>.p8
set -eu

PICO8="${PICO8_BIN:-/Applications/PICO-8.app/Contents/MacOS/pico8}"
PROBE="$1"
STEM=$(basename "$PROBE" .p8)
GOLDEN_DIR="$(dirname "$0")/../tests/conformance/golden"
GOLDEN="$GOLDEN_DIR/$STEM.txt"
HOME_DIR=$(mktemp -d)
# Some cart operations (e.g. cstore()) make PICO-8 treat the cart as
# "dirty" and rewrite it to disk on exit (version bump, appended default
# sections) even under -x. Run a scratch copy, never the committed probe
# file itself, so that side effect can't mutate tracked source.
SCRATCH_CART="$HOME_DIR/$(basename "$PROBE")"

if [ ! -x "$PICO8" ]; then
    echo "official PICO-8 binary not found/executable at $PICO8" >&2
    exit 1
fi

mkdir -p "$GOLDEN_DIR"
cp "$PROBE" "$SCRATCH_CART"

# alarm-guard: coreutils `timeout` isn't installed on this machine.
perl -e 'alarm 25; exec @ARGV' "$PICO8" -home "$HOME_DIR" -x "$SCRATCH_CART" \
    | grep -E '^(RESULT|DONE)' > "$GOLDEN"

rm -rf "$HOME_DIR"
echo "wrote $GOLDEN"
