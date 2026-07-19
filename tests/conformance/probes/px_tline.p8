pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: tline() textured lines over poked sprite+map data
cls()
for a=0,0x0fff do poke(a,(a*7+a\64)%256) end
for x=0,15 do for y=0,15 do mset(x,y,(x+y*3)%64) end end
-- horizontal, vertical, diagonal walks in map space
tline(0,0,60,0,0,0,1/8,0)
tline(0,4,60,4,0,0,1/4,0)
tline(0,8,0,60,0,0,0,1/8)
tline(4,8,40,50,0,0,1/8,1/8)
-- default mdx/mdy (1/8,0)
tline(0,70,50,70,2,3)
-- fractional start coords + negative steps
tline(20,80,90,86,1.5,2.25,-1/8,1/16)
-- layer mask arg
fset(5,0b1)
tline(0,100,80,100,0,0,1/8,0,0b1)
-- pen offset register 0x5f3b..? none; wrap behavior of map coords
tline(0,110,100,110,15.5,0,1/2,0)
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
