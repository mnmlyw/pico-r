pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- The `@`/`%`/`$` peek-shortcuts' address-expression extraction didn't stop
-- before `&`/`|`/`^^` (dialect sugar for band/bor/bxor, rewritten to
-- function calls in this same pass), so `@addr^^mask` swallowed the whole
-- `addr^^mask` as the peek() address instead of applying the bxor to the
-- peek() result -- confirmed against official PICO-8 with a diagnostic
-- probe that poked distinguishable values at both the "tight" and "loose"
-- candidate addresses (dontdig-1.p8.png: `poke(24374,@24374^^32)`).
-- Plain `^` (real Lua exponentiation, not dialect sugar) is confirmed to
-- behave the opposite way -- `@addr^2` really means `peek(addr^2)`, so it
-- must NOT be added to the same stop set.
poke(0x4300,0x0f)
printh("RESULT peek_or "..(@0x4300|0xf0))

poke(0x4300,0xff)
printh("RESULT peek_and "..(@0x4300&0x0f))

poke(0x4300,5)
printh("RESULT peek_bxor "..(@0x4300^^6))

poke(9,55)
poke(0x4300,3)
printh("RESULT peek_exp "..(@0x4300^2==peek(0x4300^2) and 1 or 0))

printh("DONE")
