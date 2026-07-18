pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- A closure must capture the NEWEST binding of a shadowed local name.
-- The old capture pushed duplicates in declaration order and lookup took
-- the first match, so a closure over `local n={...}` declared after a
-- param `n` bound the stale param (driftmania-5.p8.png:
-- `n.B=function()return nB(n)end` captured the constructor's numeric
-- argument instead of the object table).
local function make(n)
 local n={v=n*10}
 n.get=function() return n.v end
 return n
end
local o=make(7)
printh("RESULT shadowed "..o.get())
printh("DONE")
