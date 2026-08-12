# Differential frame harness

Runs a corpus cart on **both** the official PICO-8 binary and pico-r with an
injected shim that prints a hash of the whole screen every frame, then reports
the first frame where the two disagree.

This exists because the exit-code sweep (`tests/conformance/sweep_baseline.tsv`)
is a dead signal: it sits at 187/188 and only observes a cart's exit code plus
a count of nonzero screen bytes in the *final* frame. A cart can exit 0, draw a
plausible-looking screen, and still have been drawing something different from
the real console since frame 3. That is invisible to the exit-code sweep and to
every hand-written probe nobody thought to write.

```sh
tools/diffharness/run-one.sh "$HOME/Library/Application Support/pico-8/bbs/carts/ascent-0.p8.png" 20
tools/diffharness/sweep.sh /tmp/diffsweep.tsv 20
```

Needs a local PICO-8 licence and the BBS cart corpus, so like `tools/oracle.sh`
it never runs in CI.

## How it works

1. `p8png2p8.py` reconstructs a `.p8` from the `.p8.png` (PNG steganography for
   the ROM sections, `dump-raw` for the Lua) so the *same* instrumented source
   can be fed to both engines.
2. `instrument.py` injects the shim (see its docstring for why each hook is
   where it is).
3. Each engine runs it; the per-frame `HF <n> <hash> <sum> <nonzero>` lines are
   compared frame by frame.

## What a verdict means

| verdict | meaning |
|---|---|
| `match` | every frame both engines produced was identical |
| `diverge` | both ran; the reported frame is the first that differed |
| `no-official` | official emitted no frames — see the token-limit note below, a load error, or a cart that never reaches a flip |
| `no-picor` | pico-r emitted no frames |

## Limitations — read before trusting a result

- **`diverge` is a lead, not a bug.** It says the screens differ, not why. Some
  divergences will be harness artifacts rather than engine bugs, and nothing
  here attributes a divergence to a root cause. Triage each one.
- **Only the no-input path is exercised.** `btn`/`btnp` are pinned to `false`,
  because the two engines disagree about resting input state. Everything behind
  a keypress in all 188 carts is unexercised, so a `match` covers much less of a
  game than it looks like it does.
- **The cart's own RNG seeding is destroyed on purpose.** Official seeds from
  wall-clock at boot, so `srand(1)` is pinned and `srand` stubbed out. Two
  consecutive official runs of an unpinned cart genuinely differ. A handful of
  carts stay nondeterministic even so (candidates: `srand(stat(...))`, `stat(1)`
  CPU reads) — those are outside this method.
- **The hash can collide.** It is a rotate-xor over `peek4`, paired with a plain
  sum and a nonzero count to cut the odds, but "these frames match" is an upper
  bound on agreement, not proof.
- **The `.p8` round-trip is not fully verified.** The gfx/map/gff/Lua sections
  were checked by round-tripping through pico-r; the `__sfx__`/`__music__`
  encoders were not checked against the official binary. Worse, pico-r's `.p8`
  text loader currently writes the SFX header to the wrong end of each entry
  (a known, separate bug), so a cart that reads SFX RAM *as data* can diverge
  for that reason alone rather than because of anything this harness is testing.
- **The 8192-token limit costs 29 carts, and shrinking the shim does not buy
  them back.** Of the 47 `no-official` carts in the checked-in baseline, 29 are
  ones pico-r ran fine; official rejects them with `program too large` once the
  shim is added. They are not near-misses that a leaner shim would recover: a
  ~20-token stub loads, the real shim does not, and a stripped variant (hash
  only, no sum or nonzero count) was measured against them and still failed on
  most — 1 recovery in the first 8 tested. Reaching these carts needs a
  fundamentally cheaper channel, not a smaller shim. The most promising is
  `extcmd("screen")`: official already writes a composited PNG with it, so a
  near-zero-token shim would suffice if pico-r implemented the same extcmd and
  the comparison moved to PNGs. Not attempted here.
- **pico-r's main loop does not call the Lua-global `flip`.** Official's does,
  after `_draw`, looked up fresh each frame — confirmed against the binary,
  including honouring a mid-run swap of `flip`. That is a real divergence in its
  own right (a cart may legally override `flip`); it is not fixed here because
  the hosts advance `frame_count`/`elapsed_time` themselves, so routing frames
  through `api_flip` would double-advance time. The shim hooks `_draw` as well
  as `flip` precisely to work around it.
