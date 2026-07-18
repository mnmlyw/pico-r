pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- extract_rhs's "glued statements" heuristic (`x=1y=2` with no space) scans
-- backward from an `=` sign looking for what could be a new statement's LHS
-- identifier -- but accepted a bare digit as a valid identifier start, so a
-- RHS like `h%1==0and 1or 0` (a modulo feeding a `==` comparison feeding an
-- and/or ternary) got mis-cut right after the `%`, leaving `1==0 and 1 or 0`
-- dangling outside the parens as broken syntax. A Lua identifier can never
-- start with a digit, so this must not treat a numeral as a new statement's
-- LHS. Confirmed against a real corpus cart (pico1karena-0.p8.png:
-- `a[i]+=h%1==0and 1or 0`).
local h=4
local a={0,0,0}
a[1]+=h%1==0 and 1 or 0
printh("RESULT modulo_ternary_compound "..a[1])

printh("DONE")
