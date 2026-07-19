pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
local n0 = select('#', peek(0,0))
local nneg = select('#', peek(0,-3))
local n1 = select('#', peek(0,1))
local n2 = select('#', peek(0,2))
printh("RESULT peek_count_n0 "..n0)
printh("RESULT peek_count_nneg "..nneg)
printh("RESULT peek_count_n1 "..n1)
printh("RESULT peek_count_n2 "..n2)
printh("DONE")
