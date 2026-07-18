pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- extract_bitwise_rhs (used by `&`/`|`/etc's RHS extraction) treated a
-- bare `~` as an unconditional stop character, presumably meant to guard
-- against swallowing a following `~=` comparison -- but `~` is ALSO
-- PICO-8's unary bitwise-NOT prefix (`~x` -> `bnot(x)`), and when it's the
-- very first character of the RHS (`a & ~b`), the stop fired immediately,
-- producing an empty RHS and leaving the whole `&` unconverted. Confirmed
-- against a real corpus cart (donsol8_v1-14.p8.png: `mouseb & ~mouseb_last`).
local a=5
local b=3
local c = a & ~b
printh("RESULT band_unary_not "..c)

local d = a | ~b
printh("RESULT bor_unary_not "..d)

printh("DONE")
