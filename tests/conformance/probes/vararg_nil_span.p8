pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
local function f(...) return {...} end
local t=f(nil,nil,49,nil,1,nil,nil,"x")
printh("RESULT len "..#t)
printh("RESULT t3 "..tostr(t[3]).." t5 "..tostr(t[5]).." t8 "..tostr(t[8]))
local a,b,c,d,e,g,h,i=unpack(t)
printh("RESULT unp "..tostr(c).." "..tostr(e).." "..tostr(i))
printh("DONE")
