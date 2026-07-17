# Conformance ledger

Gaps found by diffing pico-r's behavior against the official PICO-8 binary
(see `tools/oracle.sh`). Each fixed/confirmed gap has a probe cart in
`probes/` and a golden capture in `golden/`; `cargo test --test conformance`
keeps them from regressing.

## Fixed

| id | subsystem | root cause | fix location |
|---|---|---|---|
| print-shorthand gluing | preprocessor | `?` (print shorthand) was only recognized at true line-start and copied its argument text raw, bypassing every other transform (string-escape repair, bitwise ops, etc). Rewritten as part of the main per-character transform loop so it fires anywhere a bare `?` appears and its argument text gets fully transformed. Confirmed against the oracle that `?`'s args run to the **end of the physical line** unconditionally (comments excepted) — a trailing `end`/`;`/keyword on the same line is swallowed and fails to compile in official PICO-8 too, not a pico-r gap. | `src/preprocessor.rs` (`process_line`) |
| P8SCII control-code string escapes | preprocessor / lexer | `\^`, `\#`, `\-`, `\|`, `\+` inside string literals were falling through the lexer's escape table (`invalid escape` lex error). Verified via oracle probe (`ord(sub(s,i,i))`) that each collapses to exactly one control byte (0x06/0x02/0x03/0x04/0x05) with the following character passed through untouched — the existing `gfx.rs` P8SCII renderer already implements those control bytes, so this was purely a compile-time encoding gap. | `src/preprocessor.rs` (`p8scii_control_escape`) |
| `extract_lhs` string-call sugar | preprocessor | `\` (int-div) and other in-place operators use `extract_lhs` to find their left operand by scanning already-emitted output backward. It didn't recognize Lua's paren-less string-call sugar (`rnd"32"`), so `rnd"32"\1` failed to transform. | `src/preprocessor.rs` (`extract_lhs`) |
| `extract_lhs` unbalanced subscript | preprocessor | Same helper treated bare `[`/`]` as "keep extending" without depth tracking, so `tq[o\64+1]` incorrectly walked back through the *open* `[` into the array name (`tq[o` instead of `o`), corrupting the subscript. Now depth-tracked like `(`/`)`. | `src/preprocessor.rs` (`extract_lhs`) |
| `?` swallowed by expression-boundary scanners | preprocessor | `extract_rhs`/`extract_simple_expr`/`extract_bitwise_rhs` (used by compound-assign and bitwise-op transforms) didn't know `?` always starts a new statement, so e.g. `sum+=0 ?"x"` had the `?"x"` clause folded into the compound-assign's RHS. Added `?` as a hard stop, matching `;`. | `src/preprocessor.rs` (three `extract_*` helpers) |
| `printh` was a no-op | api | Needed for this whole harness to work at all — now writes to stdout on native builds via a shared `display_string` helper (also used by `tostr`). | `src/pico_lua/api.rs` |

Corpus impact (188 real BBS carts, 120 frames each, `tests/conformance/sweep_baseline.tsv`):
**91→100 clean, 79→70 load errors, zero regressions.**

## Deferred / newly discovered (not fixed this batch)

- **`try_bitwise_op`'s RHS splice doesn't recurse.** Unlike compound-assign's RHS (which recursively calls `preprocess()` on its captured span), `try_bitwise_op` copies its captured RHS verbatim from the untransformed source. A backslash nested inside a bitwise RHS (e.g. `e&tq[o\64+1]`, from `picodex_dual-1.p8.png`) still isn't transformed. This is a broader "extracted spans don't recurse" pattern that likely also affects `try_peek_shortcut` — worth its own investigation rather than a one-off patch.
- **`tostr(n, 0x1)` (hex fixed-point format flag) unimplemented** — returns `"1.5"` instead of `"0x0001.8000"`. Found via the smoke probe.
- **`_set_fps` global function missing.** Confirmed via oracle to be a real official PICO-8 API (`pico1kmatch-0.p8.png` calls it at the top level); pico-r has no equivalent.
- **`print()` pixel-level rendering diverges from official PICO-8 even for the trivial case** (`cls(0); print("a")` produces a different screen-byte hash). This is unrelated to the preprocessor work in this batch — likely a default text color, cursor position, or glyph-bitmap difference — and needs its own dedicated investigation before any print-shorthand test can safely assert on screen pixels. All probes in this batch use `printh`/computed values instead of screen hashes to avoid being confounded by it.
- The pre-existing README "Known limitations" gaps (coroutines/`flip`, true 16:16 fixed-point, `string.find/match/gmatch/gsub`) remain unaddressed — architectural-scale, out of scope for a preprocessor-level batch.
- Remaining `LOAD_ERROR` carts in the corpus (70) include a distinct compound-assign bug independently discovered: `obj.rem[axis] = obj.rem[axis] + (axis=="x" and) ox or oy` (`celesteprogrupter-2.p8.png`) — the RHS extraction for `+=` splits a parenthesized `and...or` ternary-style expression incorrectly. Not investigated further this batch.
