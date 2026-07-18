pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Does a numeric-for loop terminate correctly when its counter would
-- overflow the 16.16 range mid-loop (e.g. incrementing past 32767)?
local n=0
for a=32760,32767 do
  n=n+1
end
printh("RESULT count_to_max "..n)

local n2=0
for a=32766,32768 do
  n2=n2+1
end
printh("RESULT count_past_max "..n2)
printh("DONE")
