pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Real Lua 5.2 `_ENV` scoping: a local/parameter/loop-variable named
-- `_ENV` redirects every free-name access in its scope through that
-- table (respecting __index, so a metatable chaining to a captured
-- top-level _ENV still reaches builtins -- the idiom real OOP-style
-- carts use, since official PICO-8 has no _G at all: tostr(_G) is
-- "[nil]", also locked in below). Confirmed real corpus carts:
-- praxis_fighter_x-2.p8.png (`function lk(_ENV)`, `for _ENV in all(n)`),
-- redash-7.p8.png (`local _ENV=....holding`).
printh("RESULT no_g "..tostr(_G))

local outer=_ENV
obj={x=5,y=7}
setmetatable(obj,{__index=outer})
function move(_ENV)
 x+=10
 y=x+y
end
move(obj)
printh("RESULT param_env "..obj.x.." "..obj.y)

t2={v=1}
setmetatable(t2,{__index=outer})
do
 local _ENV=t2
 v+=100
 w=42
end
printh("RESULT do_env "..t2.v.." "..tostr(t2.w))

for _ENV in all({obj}) do
 x+=1
end
printh("RESULT for_env "..obj.x)
printh("DONE")
