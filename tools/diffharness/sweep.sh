#!/usr/bin/env bash
# Run the frame-hash differential over the whole BBS cart corpus.
#
# Usage: sweep.sh <out.tsv> [frames] [workdir]
#
# Emits one row per cart from run-one.sh. This is the signal the exit-code
# sweep cannot give: a cart can exit 0 with a plausible-looking screen and
# still be drawing something different from the real console on frame 3.
set -uo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${1:?usage: sweep.sh <out.tsv> [frames] [workdir]}"
FRAMES="${2:-20}"
W="${3:-/tmp/diffharness}"
CARTS="$HOME/Library/Application Support/pico-8/bbs/carts"
LOG="${OUT%.tsv}.log"

: > "$OUT"
: > "$LOG"
n=0
total=$(ls "$CARTS"/*.p8.png 2>/dev/null | wc -l | tr -d ' ')

for cart in "$CARTS"/*.p8.png; do
  n=$((n + 1))
  row=$("$REPO/tools/diffharness/run-one.sh" "$cart" "$FRAMES" "$W" 2>/dev/null)
  echo "$row" >> "$OUT"
  echo "[$n/$total] $row" >> "$LOG"
done

echo "DONE" >> "$LOG"
