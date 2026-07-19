pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: spr/sspr edge cases -- fractional sizes, negative dest
-- sizes (flips), zero sizes, offscreen source rects, map layer combos
cls()
for a=0,0x0fff do poke(a,(a*7+a\64)%256) end
for x=0,15 do for y=0,15 do mset(x,y,(x*5+y)%64) end end
fset(2,0b01) fset(7,0b10) fset(12,0b11)
-- fractional spr sizes
spr(1,0,0,0.5,1)
spr(1,8,0,1,0.25)
spr(1,20,0,1.9,1.9)
spr(1,40,0,0.1,0.1)
-- negative and zero dest sizes in sspr
sspr(0,0,8,8,50,0,-8,8)
sspr(0,0,8,8,60,0,8,-8)
sspr(0,0,8,8,70,0,-8,-8)
sspr(0,0,8,8,80,0,0,8)
sspr(0,0,8,8,84,0,8,0)
-- source rect hanging off the sheet
sspr(120,0,16,8,90,0,16,8)
sspr(-4,0,8,8,110,0,8,8)
-- sspr fractional args (should truncate?)
sspr(0.9,0.9,8.9,8.9,0,20,8.9,8.9)
-- map layer combos
map(0,0,0,40,6,2,0b01)
map(0,0,0,60,6,2,0b10)
map(0,0,0,80,6,2,0b11)
map(6,0,60,40,4,2)
-- negative screen coords
spr(3,-4,100)
map(0,0,-8,110,3,2)
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
