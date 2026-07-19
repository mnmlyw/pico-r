pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
cls()
rectfill(-0.5,-0.5,5.5,5.5,7)
pset(20.5,-0.5,8)
pset(-0.5,20.5,9)
circfill(-0.5,50,3,10)
circfill(20,20,-0.5,7)
line(-0.5,60,-0.5,70,11)
oval(-0.5,80,10.5,90.5,12)
spr(1,-0.5,10)
sspr(0,0,8,8,50,-0.5,16,16)
local hexd="0123456789abcdef"
for r=0,127 do
 local s=""
 for b=0,63 do
  local v=peek(0x6000+r*64+b)
  local hi=v\16
  local lo=v%16
  s=s..sub(hexd,hi+1,hi+1)..sub(hexd,lo+1,lo+1)
 end
 printh("RESULT r"..r.." "..s)
end
printh("DONE")
