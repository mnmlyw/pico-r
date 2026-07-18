pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- extract_rhs's "space before an identifier that isn't and/or/not means
-- a new statement starts here" heuristic didn't check whether the WORD
-- immediately before that space was itself and/or/not -- so a compound
-- assign whose RHS is an and/or ternary (very common PICO-8 idiom,
-- e.g. `x+=cond and a or b`) split right after "and", leaving the
-- right-hand operand dangling outside the parens. Confirmed against a
-- real corpus cart (celesteprogrupter-2.p8.png:
-- `obj.rem[axis]+=axis=="x" and ox or oy`).
local axis="x"
local rem=0
rem+=axis=="x" and 5 or 9
printh("RESULT ternary_rhs "..rem)

local x=0
x+=1 and 2 or 3
printh("RESULT and_rhs "..x)

printh("DONE")
