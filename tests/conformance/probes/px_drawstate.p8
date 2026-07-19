pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: draw-state registers -- current-color fallback, color()
-- return, high color bits, 0x5f34 fillp-from-color mode, palt high bits
cls()
color(9)
rectfill(0,0,10,5)
pset(12,2)
line(14,0,20,5)
circfill(26,3,3)
-- color arg > 15: low nibble draws, high nibble = fill pattern color
rectfill(30,0,40,5,0x4e)
pset(43,2,0x21)
-- 0x5f34 bit 0: color() carries fillp+transparency in high bits
poke(0x5f34,1)
rectfill(46,0,80,10,0x33c8.5)
circfill(90,5,5,0x114e)
poke(0x5f34,0)
-- draw color register persists across calls
rectfill(84,12,90,14)
-- palt via poke: draw-pal high bit marks transparency
for a=0,63 do poke(a,0x10*(a%4)+(a%4)) end
poke(0x5f01,0x11)
poke(0x5f02,0x82)
spr(0,0,20)
pal()
-- line() invalid-state: single-arg line after cls draws nothing
cls(1)
line(20,30)
line(30,30,40,35,7)
line(50,38)
-- circ negative radius, huge radius clip
circ(64,50,-2,8)
circfill(64,60,200,2)
rectfill(0,70,127,80,3)
function dump()
 local h="0123456789abcdef"
 for r=0,127 do
  local s=""
  for b=0,63 do
   local v=peek(0x6000+r*64+b)
   s=s..sub(h,v\16+1,v\16+1)..sub(h,v%16+1,v%16+1)
  end
  printh("RESULT r"..(r<10 and "00" or (r<100 and "0" or ""))..r.." "..s)
 end
 printh("DONE")
end
dump()
