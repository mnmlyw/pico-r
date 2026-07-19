pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: pal/palt remaps, clip regions, camera offsets
cls()
rectfill(0,0,20,10,8)
pal(8,11)
rectfill(24,0,44,10,8)
pal(9,140,1)
rectfill(48,0,68,10,9)
pal()
rectfill(72,0,92,10,8)
-- palt: draw sprite with custom transparency
for a=0,63 do poke(a,0x10*(a%3)+(a%3)) end
palt(0,false)
palt(1,true)
spr(0,100,0)
palt()
spr(0,110,0)
-- clip
clip(10,20,30,10)
rectfill(0,16,127,34,12)
clip()
rectfill(0,36,10,40,13)
clip(60,20,20,10)
clip(65,22,40,40)
rectfill(0,16,127,60,14)
clip()
-- camera
camera(-20,-50)
pset(0,0,7)
rectfill(2,2,12,8,9)
line(0,10,20,10,10)
camera(10,20)
circfill(30,80,5,11)
camera()
pset(64,120,15)
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
