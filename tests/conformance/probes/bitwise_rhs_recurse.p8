pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- try_bitwise_op's captured RHS was copied verbatim from the
-- untransformed source, bypassing every other preprocessor transform --
-- so a binary literal or backslash int-div nested inside a bitwise
-- expression's RHS never got converted, and the raw text (e.g.
-- `0b11010011`) reached the real Lua lexer, which can't parse it.
local s=5
local ok = fget(s) & 0b11010011 > 0
printh("RESULT binary_literal_rhs "..tostr(ok))

local e,o=1,2
local tq={0,0,0}
local x = e&tq[o\64+1]
printh("RESULT nested_intdiv_rhs "..x)

printh("DONE")
