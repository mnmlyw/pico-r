pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: spr/sspr/map from PROGRAMMATICALLY poked gfx+map data
cls()
-- deterministic sprite sheet: sprite n gets pattern bytes derived from addr
for a=0,0x07ff do
 poke(a,(a*7+a\64)%256)
end
-- map cells
for i=0,15 do
 mset(i,0,i)
 mset(i,1,(i*3)%32)
end
-- flags for layer test
fset(3,0,true)
fset(5,0b10)
spr(1,0,0)
spr(2,10,0,2,1)
spr(3,30,0,1,1,true)
spr(4,40,0,1,1,false,true)
spr(5,50,0,1.5,0.5)
spr(17,60,0)
sspr(8,0,8,8,70,0,16,16)
sspr(0,0,16,8,90,0,8,4)
sspr(8,8,8,8,100,0,8,8,true,true)
sspr(4,4,7,3,110,0,14,9)
map(0,0,0,20,8,2)
map(0,0,0,40,16,1,0b1)
map(8,0,64,40,4,1)
mset(200,40,7)
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
