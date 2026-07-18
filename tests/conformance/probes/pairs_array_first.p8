pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pairs()/next() should yield sequential positive-integer ("array part")
-- keys first, in ascending order, before any hash-keyed entries -- this
-- matches real Lua/PICO-8 for the common list-like-table case (exact
-- hash-part order for non-array keys is a separate, deliberately
-- unmatched gap; see LEDGER.md).
local t={}
t[3]="c"
t.name="x"
t[1]="a"
t[2]="b"
t.other="y"
local order=""
for k,v in pairs(t) do
  if type(k)=="number" then
    order=order..k..","
  end
end
printh("RESULT array_order "..order)
printh("DONE")
