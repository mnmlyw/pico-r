pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Binary `~` is XOR (Lua-5.3 style; == `^^`), confirmed via oracle
-- (5~3==6); when nothing value-like precedes it, `~` stays unary bnot.
-- Real corpus carts: brandgnume-1.p8.png, kokoroko-3.p8.png.
-- inext(t,i) is the sequence-indexed iterator builtin (kalikan_stage_1b).
-- (`~0`'s "-0" display is the known deferred fixed-point gap -- not
-- asserted here.)
printh("RESULT binary_tilde "..(5~3))
printh("RESULT tilde_unary_5 "..(~5))
local t={10,20,30}
local acc=""
for i,v in inext,t do acc=acc..i..":"..v.." " end
printh("RESULT inext "..acc)
printh("DONE")
