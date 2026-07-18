pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- `#` (and unpack(), which uses it) follows Lua's luaH_getn border
-- search, spanning interior nil holes -- NOT first-hole semantics.
-- All shapes oracle-confirmed. Real corpus cart: fakogejuzo-0.p8.png
-- destructures containers holding nil fields via unpack(self).
local t={1,2,nil,4,5}
printh("RESULT len "..#t)
local a,b,c,d,e=unpack(t)
printh("RESULT unpack "..tostr(a).." "..tostr(b).." "..tostr(c).." "..tostr(d).." "..tostr(e))
local u={}
u[1]=10 u[2]=20 u[4]=40 u[5]=50
printh("RESULT sparse_len "..#u)
local m={}
m[1000]=1
printh("RESULT big_key_only "..#m)
local w={1,2,3}
w[1000]=1
printh("RESULT seq_plus_big "..#w)
local z={}
z[3]=1
printh("RESULT hole_start "..#z)
printh("DONE")
