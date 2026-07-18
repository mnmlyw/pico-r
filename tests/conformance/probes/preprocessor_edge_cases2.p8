pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- 1. `!=` after a bitwise op's RHS: `!` must be a hard stop or
--    `btn()&15!=x` captures `15!` into the rewritten band() and the
--    comparison never converts. (deepening-0.p8.png)
local last=0
if 3&15!=last then
 printh("RESULT band_neq ok")
end

-- 2. A `%` preceded by whitespace after a value is still modulo, not the
--    peek2-shortcut -- Lua is whitespace-insensitive between tokens.
--    (deepening-0.p8.png: `if timer %2==0 then`)
local timer=4
if timer %2==0 then
 printh("RESULT spaced_modulo ok")
end

-- 3. A short-if-looking line whose "body" starts with a binary operator
--    (operator AND operand, not just a lone dangling operator) is a
--    multi-line condition, not a short-if. (deepening-0.p8.png:
--    `if (abs(en.x-64))>80` / next line `or ... then`)
local en=100
if (abs(en - 64))>80
	or en<0 then
 printh("RESULT multiline_cond nomatch")
else
 printh("RESULT multiline_cond match")
end

-- 4. tonum() on booleans: true->1, false->0 (not nil). Carts use
--    `tonum(btn"1")-tonum(btn"0")` for directional input.
printh("RESULT tonum_bools "..tonum(true).." "..tonum(false))

printh("DONE")
