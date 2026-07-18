pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Numeric-for loop counter accumulation: real PICO-8 accumulates the loop
-- variable as exact 16.16 fixed steps, so a fractional, non-power-of-two
-- step run for many iterations never drifts. Plain f64 += would drift.
local i
local n=0
for j=0,100,0.1 do
  i=j
  n=n+1
end
printh("RESULT last_i "..i)
printh("RESULT iter_count "..n)

local k
local m=0
for j=0,10,1/3 do
  k=j
  m=m+1
end
printh("RESULT last_k "..k)
printh("RESULT iter_count2 "..m)
printh("DONE")
