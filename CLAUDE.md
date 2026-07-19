# Working in this repo

pico-r is a PICO-8 emulator. Its whole value proposition is behavioral
fidelity to the real console, so the rules here are about how that fidelity
is established and protected — read `README.md`'s "Conformance testing"
section first for the mechanics (`tools/oracle.sh`, `tests/conformance/`,
`LEDGER.md`).

## The official PICO-8 binary is the only ground truth

Never resolve a behavioral question from the wiki, the manual, or general
Lua knowledge when it conflicts with what the real console does. Docs are
frequently wrong or incomplete about PICO-8's actual quirks (see `LEDGER.md`
for examples where "obviously correct" reasoning from docs was wrong). If
you have access to the binary, use `tools/oracle.sh` against a minimal probe
cart to settle the question empirically.

**If you don't have access to the binary** (this is the normal case in CI —
the binary is licensed/non-redistributable and never present there, and
neither is the BBS cart corpus used for regression sweeps): work only from
the probe/golden pairs already checked into `tests/conformance/`. Do not
guess at new behavior and do not edit an existing golden file to make your
code pass — a golden is a captured fact about the real console, not a test
fixture you get to adjust.

## Fixing a gap in `tests/conformance/expected_failures.txt`

Each line names a probe with a real, oracle-confirmed divergence that
isn't fixed yet (see the file's own comment and each probe's inline
comments for what's being tested — they're written to explain the
hypothesis, not just assert an output).

1. Read the probe and its golden. Understand *why* the real console
   produces that output before touching engine code.
2. Fix the engine, not the probe or the golden.
3. `cargo test --release --test conformance` must show the specific
   probe(s) you're working passing.
4. Remove *only* the line(s) for probes you actually fixed from
   `expected_failures.txt` — leave every other line alone.
5. `cargo fmt` and `cargo clippy --release --all-targets` must be clean.
6. Add an entry to `tests/conformance/LEDGER.md` describing the root
   cause and the fix, in the file's existing style.

**Do not fabricate a fix.** A fix must be a real rule that generalizes
(the actual rasterization algorithm, the actual parsing rule, the actual
memory semantics) — never a lookup table or special-cased branch keyed on
the exact values in the probe's golden. If you can't derive the real rule,
leave the probe in `expected_failures.txt` and say what you tried and
where you got stuck, rather than reproducing the golden's output by
construction. A "passing" fix that only matches this one probe by
coincidence is worse than an honest open gap — it hides the bug instead of
fixing it, and the corpus sweep (see below) will eventually expose it
anyway, at which point it's harder to find.

## What CI can and can't verify

`cargo test --test conformance` here only checks already-captured goldens
byte-for-byte — it needs no PICO-8 install and is safe to run anywhere.
It is **not** the full verification bar this project uses before merging
to `main`: a full regression sweep over ~190 real BBS-downloaded carts
(checking for zero exit-code regressions) is required too, and that sweep
needs the actual cart files and a local PICO-8 license, neither of which
exist in CI. Land fixes as a PR rather than pushing directly to `main` so
that step can happen before merge.

## Style

No comments explaining *what* code does or referencing the task/history
that produced it — only comments that capture a non-obvious *why* (a
hidden constraint, an oracle-confirmed quirk that looks like a bug,
a workaround with a specific cause). This matches the existing codebase;
look at any function in `src/pico_lua/api.rs` or `src/gfx.rs` for the
tone to match.
