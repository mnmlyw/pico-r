#!/usr/bin/env bash
# Run one corpus cart on both engines with the frame-hash shim injected and
# report the first frame at which they disagree.
#
# Usage: run-one.sh <cart.p8.png> [frames] [workdir]
# Prints one TSV row: <stem> <verdict> <first-diff-frame> <official-frames> <picor-frames>
#
# Verdicts:
#   match        every frame both produced is identical
#   diverge      both ran, but a frame differs (first-diff-frame says which)
#   no-official  official produced no frames (over the token limit, a load
#                error, or a cart that never reaches a flip)
#   no-picor     pico-r produced no frames
set -uo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
PICO8="${PICO8_BIN:-/Applications/PICO-8.app/Contents/MacOS/pico8}"
CART="$1"
FRAMES="${2:-20}"
W="${3:-/tmp/diffharness}"
STEM=$(basename "$CART" .p8.png)

mkdir -p "$W/home" "$W/carts" "$W/out"

# .p8.png -> .p8 so the same instrumented source can be fed to both engines.
"$REPO/target/release/dump-raw" "$CART" >/dev/null 2>&1 || { echo -e "$STEM\tno-picor\t0\t0\t0"; exit 0; }
python3 "$REPO/tools/diffharness/p8png2p8.py" "$CART" /tmp/raw_lua.lua "$W/carts/$STEM.p8" >/dev/null 2>&1 \
  || { echo -e "$STEM\tno-official\t0\t0\t0"; exit 0; }
python3 "$REPO/tools/diffharness/instrument.py" "$W/carts/$STEM.p8" "$W/carts/${STEM}_i.p8" "$FRAMES" \
  || { echo -e "$STEM\tno-official\t0\t0\t0"; exit 0; }

# Official stops itself via extcmd("shutdown") once it has emitted FRAMES
# frames; the alarm is only a backstop for carts that never get there.
perl -e 'alarm 120; exec @ARGV' "$PICO8" -x "$W/carts/${STEM}_i.p8" -home "$W/home" 2>/dev/null \
  | grep '^HF ' > "$W/out/$STEM.official" || true

# pico-r ignores extcmd, so it is bounded by its own frame budget instead.
perl -e 'alarm 120; exec @ARGV' "$REPO/target/release/run-cart" "$W/carts/${STEM}_i.p8" $((FRAMES * 3)) 2>/dev/null \
  | grep '^HF ' > "$W/out/$STEM.picor" || true

no=$(wc -l < "$W/out/$STEM.official" | tr -d ' ')
np=$(wc -l < "$W/out/$STEM.picor" | tr -d ' ')
[ "$no" -eq 0 ] && { echo -e "$STEM\tno-official\t0\t$no\t$np"; exit 0; }
[ "$np" -eq 0 ] && { echo -e "$STEM\tno-picor\t0\t$no\t$np"; exit 0; }

# Compare only the frames both engines actually produced.
n=$(( no < np ? no : np ))
first=$(python3 - "$W/out/$STEM.official" "$W/out/$STEM.picor" "$n" <<'PY'
import sys
o=[l.split() for l in open(sys.argv[1])][:int(sys.argv[3])]
p=[l.split() for l in open(sys.argv[2])][:int(sys.argv[3])]
for k,(a,b) in enumerate(zip(o,p),1):
    if a[2:]!=b[2:]:
        print(k); break
else:
    print(0)
PY
)
[ "$first" -eq 0 ] && echo -e "$STEM\tmatch\t0\t$no\t$np" || echo -e "$STEM\tdiverge\t$first\t$no\t$np"
