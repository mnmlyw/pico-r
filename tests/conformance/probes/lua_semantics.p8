pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- del() during all()-iteration must not skip or double-visit items.
-- Deleting a LATER item leaves the current index's content unchanged
-- (must advance normally); deleting the CURRENT item shifts later
-- items down into this same index (must re-visit it). Tracking just
-- "did the length shrink" can't distinguish the two.
local objs={1,2,3,4}
local seen=""
for v in all(objs) do
  seen=seen..v
  if v==2 then del(objs,4) end
end
printh("RESULT del_later_seen "..seen)

local objs2={1,2,3}
local seen2=""
for v in all(objs2) do
  seen2=seen2..v
  if v==2 then del(objs2,2) end
end
printh("RESULT del_current_seen "..seen2)

local objs3={1,2,3,4,5}
local seen3=""
for v in all(objs3) do
  seen3=seen3..v
  if v%2==0 then del(objs3,v) end
end
printh("RESULT del_evens_seen "..seen3)

-- Arithmetic/concat operators must consult a table operand's metatable
-- (__add/__sub/__mul/__concat) before falling back to raw number/string
-- coercion -- the manual's own worked example redefines + for vectors.
local mt={
  __add=function(a,b) return a.x+b.x end,
  __sub=function(a,b) return a.x-b.x end,
  __mul=function(a,b) return a.x*b.x end,
  __concat=function(a,b) return "concat!" end,
}
local v1=setmetatable({x=10},mt)
local v2=setmetatable({x=3},mt)
printh("RESULT add "..tostr(v1+v2))
printh("RESULT sub "..tostr(v1-v2))
printh("RESULT mul "..tostr(v1*v2))
printh("RESULT concat "..(v1 .. v2))

printh("DONE")
