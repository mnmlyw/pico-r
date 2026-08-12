#!/usr/bin/env python3
"""Classify a diffharness sweep, separating real divergences from artifacts.

Usage: analyze.py <sweep.tsv> [workdir]

`run-one.sh` compares frame k against frame k, which reports a divergence when
the two engines are actually drawing the same sequence one frame out of step.
That is a harness artifact, not an engine bug, and it needs to be split out
before anyone spends time triaging the list.

Re-classifies each `diverge` row by testing whether official frame k+s equals
pico-r frame k for some small shift s:

  content-diff   no shift aligns them -- the engines really are drawing
                 different things. This is the triage queue.
  phase-shift    a shift lines the sequences up, so the frames themselves
                 agree and only the starting offset differs.

Also reports the nonzero-word delta at the first differing frame, which says
whether the old exit-code sweep could ever have seen this cart: a delta of 0
means identical byte counts and different content, i.e. structurally invisible
to that metric.
"""

import sys
from pathlib import Path

MIN_FRAMES_FOR_SHIFT = 5
MAX_SHIFT = 3


def load(path):
    try:
        return [line.split() for line in open(path)]
    except OSError:
        return []


def best_shift(official, picor):
    """Return (shift, matched) for the alignment matching the most frames."""
    best = (0, -1)
    for s in range(-MAX_SHIFT, MAX_SHIFT + 1):
        matched = total = 0
        for k in range(len(picor)):
            oi = k + s
            if 0 <= oi < len(official):
                total += 1
                if official[oi][2:] == picor[k][2:]:
                    matched += 1
        if total >= MIN_FRAMES_FOR_SHIFT and matched > best[1]:
            best = (s, matched)
    return best


def main():
    sweep = Path(sys.argv[1])
    work = Path(sys.argv[2] if len(sys.argv) > 2 else "/tmp/diffharness")
    out = work / "out"

    counts = {}
    content, phase = [], []

    for line in open(sweep):
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 3:
            continue
        stem, verdict, frame = parts[0], parts[1], int(parts[2])
        if verdict != "diverge":
            counts[verdict] = counts.get(verdict, 0) + 1
            continue

        o, p = load(out / f"{stem}.official"), load(out / f"{stem}.picor")
        shift, matched = best_shift(o, p)
        delta = None
        if len(o) >= frame and len(p) >= frame:
            delta = int(p[frame - 1][4]) - int(o[frame - 1][4])

        if shift != 0 and matched >= MIN_FRAMES_FOR_SHIFT:
            phase.append((stem, shift, matched))
            counts["phase-shift"] = counts.get("phase-shift", 0) + 1
        else:
            content.append((frame, stem, delta))
            counts["content-diff"] = counts.get("content-diff", 0) + 1

    for k in sorted(counts):
        print(f"  {k:<14} {counts[k]}")

    if phase:
        print("\nphase-shift (same frames, different starting offset -- artifact):")
        for stem, shift, matched in sorted(phase, key=lambda x: -x[2]):
            print(f"  {stem:<36} shift {shift:+d}  ({matched} frames align)")

    if content:
        print("\ncontent-diff (triage queue), by first differing frame:")
        invisible = 0
        for frame, stem, delta in sorted(content):
            d = "n/a" if delta is None else f"{delta:+d}"
            if delta == 0:
                invisible += 1
            print(f"  frame {frame:<4} {stem:<36} nonzero-word delta {d}")
        print(
            f"\n  {invisible}/{len(content)} have an identical nonzero-word count, "
            "so the exit-code sweep could never have seen them"
        )


if __name__ == "__main__":
    main()
