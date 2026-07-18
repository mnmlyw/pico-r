pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Unary minus dispatches to the `__unm` metamethod for non-number
-- operands, like the other arithmetic metamethods -- 3D-math carts
-- overload it on vectors (mot_pool-23.p8.png).
local mt={__unm=function(v) return {x=-v.x,y=-v.y} end}
local v=setmetatable({x=3,y=-4},mt)
local n=-v
printh("RESULT unm "..n.x.." "..n.y)
printh("DONE")
