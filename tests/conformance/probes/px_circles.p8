pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: circ/circfill/oval/ovalfill radii and degenerate cases
cls()
for r=0,8 do
 circ(8+r*14,10,r,7)
 circfill(8+r*14,30,r,8)
end
circ(10,50,2.5,9)
circ(24,50,2.49,10)
circfill(40,50,3.7,11)
circ(60,50,-1,12)
circfill(76,50,-3,13)
circ(90,50,0,14)
circfill(104,50,0,15)
-- big/offscreen circles
circ(0,80,20,1)
circfill(127,80,15,2)
circ(64,200,100,3)
-- ovals: corner form
oval(4,96,40,110,7)
ovalfill(44,96,60,124,8)
oval(70,100,70,120,9)
oval(80,100,100,100,10)
oval(110,105,104,95,11)
ovalfill(120,126,126,96,12)
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
